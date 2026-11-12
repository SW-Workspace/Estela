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
    for dy in 0..20 {
        for dx in 0..12 {
            if dx > dy / 2 {
                continue;
            }
            let (px, py) = (x + dx, y + dy);
            if px >= 0 && py >= 0 && px < size.0 as i32 && py < size.1 as i32 {
                let offset = ((py as u32 * size.0 + px as u32) * 4) as usize;
                let border = dx == 0 || dx == dy / 2 || dy == 19;
                output[offset..offset + 4].copy_from_slice(if border {
                    &[20, 20, 24, 255]
                } else {
                    &[255, 255, 255, 255]
                });
            }
        }
    }
}
