use std::{
    os::fd::OwnedFd,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::SyncSender,
        Arc, Mutex,
    },
    time::Duration,
};

use pipewire::{
    self as pw,
    properties::properties,
    spa::{
        self,
        param::{
            format::{FormatProperties, MediaSubtype, MediaType},
            video::{VideoFormat, VideoInfoRaw},
            ParamType,
        },
        pod::{ChoiceValue, Pod, Property, Value},
        utils::{Choice, ChoiceEnum, ChoiceFlags, Direction, SpaTypes},
    },
};

use estela_capture::{
    session::{Frame, Pointer},
    Error, Result,
};

const DEFAULT_CURSOR_EDGE: i32 = 64;
const MIN_CURSOR_EDGE: i32 = 1;
const MAX_CURSOR_EDGE: i32 = 1024;
const CURSOR_BYTES_PER_PIXEL: i32 = 4;

struct StreamData {
    format: VideoInfoRaw,
    sender: SyncSender<Result<Frame>>,
    pointer: Arc<Mutex<Pointer>>,
}

fn backend(error: impl std::fmt::Display) -> Error {
    Error::Backend(error.to_string())
}

pub fn capture(
    remote: OwnedFd,
    node: u32,
    fps: u32,
    sender: SyncSender<Result<Frame>>,
    pointer: Arc<Mutex<Pointer>>,
    stop: Arc<AtomicBool>,
) -> Result<()> {
    pw::init();
    let main_loop = pw::main_loop::MainLoopRc::new(None).map_err(backend)?;
    let context = pw::context::ContextRc::new(&main_loop, None).map_err(backend)?;
    let core = context.connect_fd_rc(remote, None).map_err(backend)?;
    let stream = pw::stream::StreamBox::new(&core, "Estela", properties! { *pw::keys::MEDIA_TYPE => "Video", *pw::keys::MEDIA_CATEGORY => "Capture", *pw::keys::MEDIA_ROLE => "Screen" }).map_err(backend)?;
    let data = StreamData {
        format: Default::default(),
        sender,
        pointer,
    };
    let cursor_metadata = cursor_metadata()?;
    let _listener = stream
        .add_local_listener_with_user_data(data)
        .param_changed(move |stream, data, id, parameter| {
            if id != ParamType::Format.as_raw() {
                return;
            }
            let Some(parameter) = parameter else {
                return;
            };
            if let Err(error) = data.format.parse(parameter) {
                let _ = data.sender.try_send(Err(backend(error)));
                return;
            }
            let Some(cursor) = Pod::from_bytes(&cursor_metadata) else {
                let _ = data
                    .sender
                    .try_send(Err(backend("Invalid cursor metadata")));
                return;
            };
            // PipeWire negotiates buffer metadata after fixing the video format.
            if let Err(error) = stream.update_params(&mut [cursor]) {
                let _ = data.sender.try_send(Err(backend(error)));
            }
        })
        .state_changed(|_, data, old, state| {
            if matches!(state, pw::stream::StreamState::Unconnected)
                && matches!(
                    old,
                    pw::stream::StreamState::Paused | pw::stream::StreamState::Streaming
                )
            {
                let _ = data
                    .sender
                    .try_send(Err(backend("Screen capture stream closed")));
            }
            if let pw::stream::StreamState::Error(error) = state {
                let _ = data.sender.try_send(Err(backend(error)));
            }
        })
        .process(process)
        .register()
        .map_err(backend)?;
    let format = spa::pod::object!(
        SpaTypes::ObjectParamFormat,
        ParamType::EnumFormat,
        spa::pod::property!(FormatProperties::MediaType, Id, MediaType::Video),
        spa::pod::property!(FormatProperties::MediaSubtype, Id, MediaSubtype::Raw),
        spa::pod::property!(
            FormatProperties::VideoFormat,
            Choice,
            Enum,
            Id,
            VideoFormat::BGRA,
            VideoFormat::BGRA,
            VideoFormat::BGRx,
            VideoFormat::RGBA,
            VideoFormat::RGBx
        ),
        spa::pod::property!(
            FormatProperties::VideoSize,
            Choice,
            Range,
            Rectangle,
            spa::utils::Rectangle {
                width: 1920,
                height: 1080
            },
            spa::utils::Rectangle {
                width: 1,
                height: 1
            },
            spa::utils::Rectangle {
                width: 16384,
                height: 16384
            }
        ),
        spa::pod::property!(
            FormatProperties::VideoFramerate,
            Choice,
            Range,
            Fraction,
            spa::utils::Fraction { num: fps, denom: 1 },
            spa::utils::Fraction { num: 0, denom: 1 },
            spa::utils::Fraction { num: fps, denom: 1 }
        ),
    );
    let format_bytes = serialize(Value::Object(format))?;
    let format = Pod::from_bytes(&format_bytes).ok_or_else(|| backend("Invalid stream format"))?;
    stream
        .connect(
            Direction::Input,
            Some(node),
            pw::stream::StreamFlags::AUTOCONNECT | pw::stream::StreamFlags::MAP_BUFFERS,
            &mut [format],
        )
        .map_err(backend)?;
    while !stop.load(Ordering::Acquire) {
        main_loop
            .loop_()
            .iterate(pw::loop_::Timeout::Finite(Duration::from_millis(20)));
    }
    Ok(())
}

fn cursor_metadata() -> Result<Vec<u8>> {
    let cursor = spa::pod::object!(
        SpaTypes::ObjectParamMeta,
        ParamType::Meta,
        spa::pod::Property::new(
            spa::sys::SPA_PARAM_META_type,
            Value::Id(spa::utils::Id(spa::sys::SPA_META_Cursor))
        ),
        // Compositors can advertise different fixed cursor bitmap capacities.
        Property::new(
            spa::sys::SPA_PARAM_META_size,
            Value::Choice(ChoiceValue::Int(Choice(
                ChoiceFlags::empty(),
                ChoiceEnum::Range {
                    default: cursor_metadata_size(DEFAULT_CURSOR_EDGE),
                    min: cursor_metadata_size(MIN_CURSOR_EDGE),
                    max: cursor_metadata_size(MAX_CURSOR_EDGE),
                },
            )))
        ),
    );
    serialize(Value::Object(cursor))
}

fn cursor_metadata_size(edge: i32) -> i32 {
    let headers = size_of::<spa::sys::spa_meta_cursor>() + size_of::<spa::sys::spa_meta_bitmap>();
    headers as i32 + edge * edge * CURSOR_BYTES_PER_PIXEL
}

fn serialize(value: Value) -> Result<Vec<u8>> {
    Ok(
        spa::pod::serialize::PodSerializer::serialize(std::io::Cursor::new(Vec::new()), &value)
            .map_err(backend)?
            .0
            .into_inner(),
    )
}

fn process(stream: &pw::stream::Stream, data: &mut StreamData) {
    let Some(mut buffer) = stream.dequeue_buffer() else {
        return;
    };
    if let Some(cursor) = buffer.find_meta::<spa::buffer::meta::MetaCursor>() {
        if cursor.is_valid() {
            if let Ok(mut pointer) = data.pointer.lock() {
                *pointer = Pointer {
                    x: cursor.position().x,
                    y: cursor.position().y,
                    available: true,
                    ..Default::default()
                };
            }
        }
    }
    let Some(plane) = buffer.datas_mut().first_mut() else {
        return;
    };
    let size = data.format.size();
    let (width, height) = (size.width, size.height);
    if width == 0 || height == 0 || width > 16384 || height > 16384 || plane.chunk().size() == 0 {
        return;
    }
    let offset = plane.chunk().offset() as usize;
    let stride = plane.chunk().stride();
    if stride < (width * 4) as i32 {
        return;
    }
    let Some(bytes) = plane.data() else {
        return;
    };
    let Some(end) = offset.checked_add(stride as usize * height as usize) else {
        return;
    };
    if end > bytes.len() {
        let _ = data
            .sender
            .try_send(Err(backend("PipeWire frame bounds are invalid")));
        return;
    }
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    let blue_first = matches!(data.format.format(), VideoFormat::BGRA | VideoFormat::BGRx);
    for row in bytes[offset..end].chunks_exact(stride as usize) {
        rgba.extend_from_slice(&row[..width as usize * 4]);
    }
    for pixel in rgba.as_chunks_mut::<4>().0 {
        if blue_first {
            pixel.swap(0, 2);
        }
        pixel[3] = 255;
    }
    let _ = data.sender.try_send(Ok(Frame {
        width,
        height,
        rgba,
    }));
}
