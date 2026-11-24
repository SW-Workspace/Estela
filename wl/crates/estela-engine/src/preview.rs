use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender, SyncSender},
    time::{Duration, Instant},
};

use estela_format::tracks::Recording;
use estela_media_ffmpeg::decode::Decoder;
use estela_model::{EventKind, Time};

use crate::Result;

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Play,
    Pause,
    Seek(f64),
    AutoZoom(bool),
    Stop,
}

#[derive(Debug)]
pub enum Event {
    Frame {
        rgba: Vec<u8>,
        size: (u32, u32),
        seconds: f64,
    },
    End,
    Failed(String),
}

pub struct Player {
    commands: Sender<Command>,
    pub events: Receiver<Event>,
}

impl Player {
    pub fn open(path: PathBuf, recording: Recording) -> Self {
        let (commands, receiver) = mpsc::channel();
        let (sender, events) = mpsc::sync_channel(2);
        std::thread::spawn(move || {
            if let Err(error) = playback(path, recording, receiver, &sender) {
                let _ = sender.send(Event::Failed(error.to_string()));
            }
        });
        Self { commands, events }
    }
    pub fn send(&self, command: Command) {
        let _ = self.commands.send(command);
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Stop);
    }
}

fn playback(
    path: PathBuf,
    recording: Recording,
    commands: Receiver<Command>,
    sender: &SyncSender<Event>,
) -> Result<()> {
    let scale = (960.0 / f64::from(recording.width))
        .min(540.0 / f64::from(recording.height))
        .min(1.0);
    let size = (
        (f64::from(recording.width) * scale).max(1.0) as u32,
        (f64::from(recording.height) * scale).max(1.0) as u32,
    );
    let mut decoder = Decoder::start(&path, size, 0.0)?;
    let mut seconds = 0.0;
    let mut playing = false;
    let mut dirty = true;
    let mut autozoom = true;
    let mut next_frame = Instant::now();
    let interval = Duration::from_secs_f64(1.0 / f64::from(recording.fps));
    let mut original = None;
    loop {
        while let Ok(command) = commands.try_recv() {
            match command {
                Command::Stop => return Ok(()),
                Command::Play => {
                    playing = true;
                    next_frame = Instant::now();
                }
                Command::Pause => playing = false,
                Command::AutoZoom(enabled) => {
                    autozoom = enabled;
                    dirty = true;
                }
                Command::Seek(target) => {
                    seconds = target.clamp(
                        0.0,
                        (recording.duration.0 as f64 / 1e9 - 1.0 / f64::from(recording.fps))
                            .max(0.0),
                    );
                    decoder = Decoder::start(&path, size, seconds)?;
                    original = None;
                    dirty = true;
                    next_frame = Instant::now();
                }
            }
        }
        if playing || dirty {
            if original.is_none() || playing {
                original = decoder.next_frame()?;
            }
            if let Some(original) = &original {
                let rgba = compose(
                    original,
                    size,
                    &recording,
                    Time((seconds * 1e9) as i64),
                    autozoom,
                );
                if sender
                    .send(Event::Frame {
                        rgba,
                        size,
                        seconds,
                    })
                    .is_err()
                {
                    return Ok(());
                }
                if playing {
                    seconds += 1.0 / f64::from(recording.fps);
                }
            } else {
                playing = false;
                if sender.send(Event::End).is_err() {
                    return Ok(());
                }
            }
            dirty = false;
        }
        if playing {
            next_frame += interval;
            std::thread::sleep(next_frame.saturating_duration_since(Instant::now()));
        } else {
            std::thread::sleep(Duration::from_millis(15));
        }
    }
}

pub fn compose(
    original: &[u8],
    size: (u32, u32),
    recording: &Recording,
    time: Time,
    autozoom: bool,
) -> Vec<u8> {
    let camera = estela_motion::camera::sample(
        if autozoom { &recording.zooms } else { &[] },
        &recording.events,
        time,
        (recording.width, recording.height),
    );
    let scale_x = f64::from(size.0) / f64::from(recording.width);
    let scale_y = f64::from(size.1) / f64::from(recording.height);
    let center = (camera.x * scale_x, camera.y * scale_y);
    if (camera.scale - 1.0).abs() < f64::EPSILON {
        let mut output = original.to_vec();
        if !recording.cursor_embedded {
            draw_cursor(&mut output, size, recording, time, center, camera.scale);
        }
        return output;
    }
    let mut output = vec![0; original.len()];
    for y in 0..size.1 {
        for x in 0..size.0 {
            let source_x = ((f64::from(x) - f64::from(size.0) / 2.0) / camera.scale + center.0)
                .clamp(0.0, f64::from(size.0 - 1)) as u32;
            let source_y = ((f64::from(y) - f64::from(size.1) / 2.0) / camera.scale + center.1)
                .clamp(0.0, f64::from(size.1 - 1)) as u32;
            let source = ((source_y * size.0 + source_x) * 4) as usize;
            let target = ((y * size.0 + x) * 4) as usize;
            output[target..target + 4].copy_from_slice(&original[source..source + 4]);
        }
    }
    if !recording.cursor_embedded {
        draw_cursor(&mut output, size, recording, time, center, camera.scale);
    }
    output
}

fn draw_cursor(
    output: &mut [u8],
    size: (u32, u32),
    recording: &Recording,
    time: Time,
    center: (f64, f64),
    scale: f64,
) {
    let position = recording.events.iter().rev().find_map(|event| {
        if event.t > time {
            return None;
        }
        if let EventKind::Move { x, y } = event.kind {
            Some((x, y))
        } else {
            None
        }
    });
    let Some((x, y)) = position else {
        return;
    };
    if x < 0 || y < 0 || x >= recording.width as i32 || y >= recording.height as i32 {
        return;
    }
    let x = ((f64::from(x) * f64::from(size.0) / f64::from(recording.width) - center.0) * scale
        + f64::from(size.0) / 2.0) as i32;
    let y = ((f64::from(y) * f64::from(size.1) / f64::from(recording.height) - center.1) * scale
        + f64::from(size.1) / 2.0) as i32;
    paint_arrow(output, size, (x, y));
}

const CURSOR_WIDTH: i32 = 20;
const CURSOR_HEIGHT: i32 = 29;
const CURSOR_OUTLINE: [(f32, f32); 7] = [
    (0.0, 0.0),
    (20.0, 16.0),
    (12.0, 17.0),
    (17.0, 26.0),
    (12.0, 29.0),
    (7.0, 20.0),
    (1.0, 26.0),
];
const CURSOR_FILL: [(f32, f32); 7] = [
    (2.0, 4.0),
    (15.0, 14.8),
    (9.0, 15.7),
    (14.5, 25.3),
    (12.8, 26.3),
    (7.4, 16.8),
    (2.8, 21.8),
];
const CURSOR_BORDER: [u16; 3] = [20, 20, 24];
const CURSOR_WHITE: u16 = 255;
const CURSOR_SAMPLE_OFFSETS: [(f32, f32); 4] =
    [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)];

fn paint_arrow(output: &mut [u8], size: (u32, u32), origin: (i32, i32)) {
    for y in origin.1.max(0)..(origin.1.saturating_add(CURSOR_HEIGHT)).min(size.1 as i32) {
        for x in origin.0.max(0)..(origin.0.saturating_add(CURSOR_WIDTH)).min(size.0 as i32) {
            let offset = (y as usize * size.0 as usize + x as usize) * 4;
            let pixel = &mut output[offset..offset + 4];
            let mut outline = 0;
            let mut fill = 0;
            for (sample_x, sample_y) in CURSOR_SAMPLE_OFFSETS {
                let point = (
                    (x - origin.0) as f32 + sample_x,
                    (y - origin.1) as f32 + sample_y,
                );
                if inside_arrow(point, &CURSOR_OUTLINE) {
                    outline += 1;
                    fill += u16::from(inside_arrow(point, &CURSOR_FILL));
                }
            }
            let samples = CURSOR_SAMPLE_OFFSETS.len() as u16;
            for (channel, border) in pixel.iter_mut().zip(CURSOR_BORDER) {
                *channel = ((u16::from(*channel) * (samples - outline)
                    + border * (outline - fill)
                    + CURSOR_WHITE * fill)
                    / samples) as u8;
            }
        }
    }
}

fn inside_arrow((x, y): (f32, f32), vertices: &[(f32, f32); 7]) -> bool {
    let mut inside = false;
    let mut previous = vertices[vertices.len() - 1];
    for &current in vertices {
        if (current.1 > y) != (previous.1 > y)
            && x < current.0 + (previous.0 - current.0) * (y - current.1) / (previous.1 - current.1)
        {
            inside = !inside;
        }
        previous = current;
    }
    inside
}
