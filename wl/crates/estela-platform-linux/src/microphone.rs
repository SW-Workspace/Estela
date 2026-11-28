use std::{
    cell::{Cell, RefCell},
    io::Cursor,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::SyncSender,
    },
    time::{Duration, Instant},
};

use pipewire::{self as pw, loop_::Timeout, properties::properties, spa};

use estela_capture::microphone::{Device, Packet, SAMPLE_BYTES, SAMPLE_RATE};

use crate::{Error, Result};

const POLL_INTERVAL: Duration = Duration::from_millis(20);
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(3);

pub fn devices() -> Result<Vec<Device>> {
    let (main_loop, core) = connect()?;
    let registry = core.get_registry().map_err(audio_error)?;
    let devices = Rc::new(RefCell::new(Vec::new()));
    let done = Rc::new(Cell::new(false));
    let done_callback = Rc::clone(&done);
    let devices_callback = Rc::clone(&devices);
    let pending = core.sync(0).map_err(audio_error)?;
    let _core_listener = core
        .add_listener_local()
        .done(move |id, sequence| {
            if id == pw::core::PW_ID_CORE && sequence == pending {
                done_callback.set(true);
            }
        })
        .register();
    let _registry_listener = registry
        .add_listener_local()
        .global(move |global| {
            let Some(props) = global.props else { return };
            if props.get("media.class") != Some("Audio/Source") {
                return;
            }
            if let Some(id) = props.get("node.name") {
                let name = props
                    .get("node.description")
                    .or_else(|| props.get("node.nick"))
                    .unwrap_or(id);
                devices_callback.borrow_mut().push(Device {
                    id: id.into(),
                    name: name.into(),
                });
            }
        })
        .register();
    let start = Instant::now();
    while !done.get() {
        if start.elapsed() >= CONNECTION_TIMEOUT {
            return Err(Error::Audio("PipeWire did not return audio sources".into()));
        }
        main_loop.loop_().iterate(Timeout::Finite(POLL_INTERVAL));
    }
    Ok(devices.take())
}

pub fn capture(target: Option<&str>, packets: SyncSender<Packet>, stop: &AtomicBool) -> Result<()> {
    let (main_loop, core) = connect()?;
    let mut props = properties! {
        "media.type" => "Audio",
        "media.category" => "Capture",
        "media.role" => "Production",
    };
    if let Some(target) = target {
        props.insert("target.object", target);
        props.insert("node.dont-fallback", "true");
        props.insert("node.dont-reconnect", "true");
    }
    let stream =
        pw::stream::StreamBox::new(&core, "Estela microphone", props).map_err(audio_error)?;
    let _listener = stream
        .add_local_listener_with_user_data(CaptureClock::default())
        .process(move |stream, clock| {
            while let Some(mut buffer) = stream.dequeue_buffer() {
                if let Some(pcm) = pcm_data(&mut buffer) {
                    let start = clock.packet_start(stream, pcm.len() / SAMPLE_BYTES);
                    let _ = packets.try_send(Packet {
                        start,
                        pcm: pcm.to_vec(),
                    });
                }
            }
        })
        .register()
        .map_err(audio_error)?;
    let format = pcm_format()?;
    let pod = spa::pod::Pod::from_bytes(&format)
        .ok_or_else(|| Error::Audio("Invalid PipeWire audio format".into()))?;
    stream
        .connect(
            spa::utils::Direction::Input,
            None,
            pw::stream::StreamFlags::AUTOCONNECT | pw::stream::StreamFlags::MAP_BUFFERS,
            &mut [pod],
        )
        .map_err(audio_error)?;
    run(&main_loop, &stream, stop)
}

fn pcm_data<'a>(buffer: &'a mut pw::buffer::Buffer<'_>) -> Option<&'a [u8]> {
    let data = buffer.datas_mut().first_mut()?;
    let offset = data.chunk().offset() as usize;
    let size = data.chunk().size() as usize;
    if size == 0
        || !size.is_multiple_of(SAMPLE_BYTES)
        || data
            .chunk()
            .flags()
            .contains(spa::buffer::ChunkFlags::CORRUPTED)
    {
        return None;
    }
    data.data()?.get(offset..offset.checked_add(size)?)
}

#[derive(Default)]
struct CaptureClock {
    graph_origin: Option<(i64, Instant)>,
    previous_end: Option<Instant>,
}

impl CaptureClock {
    fn packet_start(&mut self, stream: &pw::stream::Stream, frames: usize) -> Instant {
        let duration = Duration::from_secs_f64(frames as f64 / f64::from(SAMPLE_RATE));
        let arrival = Instant::now();
        let estimated = self.graph_end(stream, arrival).unwrap_or(arrival) - duration;
        let start = self
            .previous_end
            .filter(|previous| {
                estimated
                    .duration_since(*previous)
                    .max(previous.duration_since(estimated))
                    < duration / 2
            })
            .unwrap_or(estimated);
        self.previous_end = Some(start + duration);
        start
    }

    fn graph_end(&mut self, stream: &pw::stream::Stream, arrival: Instant) -> Option<Instant> {
        let time = stream.time().ok()?;
        let rate = time.rate();
        if time.now() <= 0 || rate.denom == 0 {
            return None;
        }
        let (graph_origin, origin) = *self.graph_origin.get_or_insert((time.now(), arrival));
        let elapsed = Duration::from_nanos(u64::try_from(time.now() - graph_origin).ok()?);
        let latency = time.delay().max(0) as f64 * f64::from(rate.num) / f64::from(rate.denom);
        origin
            .checked_add(elapsed)?
            .min(arrival)
            .checked_sub(Duration::try_from_secs_f64(latency).ok()?)
    }
}

fn run(
    main_loop: &pw::main_loop::MainLoopRc,
    stream: &pw::stream::Stream,
    stop: &AtomicBool,
) -> Result<()> {
    use pw::stream::StreamState;

    let start = Instant::now();
    let mut connected = false;
    while !stop.load(Ordering::Relaxed) {
        main_loop.loop_().iterate(Timeout::Finite(POLL_INTERVAL));
        match stream.state() {
            StreamState::Streaming => connected = true,
            StreamState::Error(error) => return Err(Error::Audio(error)),
            StreamState::Unconnected if connected => {
                return Err(Error::Audio("Microphone disconnected".into()))
            }
            _ if !connected && start.elapsed() >= CONNECTION_TIMEOUT => {
                return Err(Error::Audio(
                    "PipeWire could not connect to the microphone".into(),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn pcm_format() -> Result<Vec<u8>> {
    let mut format = spa::param::audio::AudioInfoRaw::new();
    format.set_format(spa::param::audio::AudioFormat::S16LE);
    format.set_rate(SAMPLE_RATE);
    format.set_channels(1);
    let object = spa::pod::Object {
        type_: spa::utils::SpaTypes::ObjectParamFormat.as_raw(),
        id: spa::param::ParamType::EnumFormat.as_raw(),
        properties: format.into(),
    };
    let (bytes, _) = spa::pod::serialize::PodSerializer::serialize(
        Cursor::new(Vec::new()),
        &spa::pod::Value::Object(object),
    )
    .map_err(audio_error)?;
    Ok(bytes.into_inner())
}

fn connect() -> Result<(pw::main_loop::MainLoopRc, pw::core::CoreRc)> {
    pw::init();
    let main_loop = pw::main_loop::MainLoopRc::new(None).map_err(audio_error)?;
    let context = pw::context::ContextRc::new(&main_loop, None).map_err(audio_error)?;
    let core = context.connect_rc(None).map_err(audio_error)?;
    Ok((main_loop, core))
}

fn audio_error(error: impl std::fmt::Debug) -> Error {
    Error::Audio(format!("{error:?}"))
}
