use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread::JoinHandle,
    time::{Duration, Instant},
};

use estela_capture::session::{Frame, Pointer, Session, Source};
use estela_format::tracks::{Recording, ZoomTrigger};
use estela_media_ffmpeg::encode::Encoder;

use crate::{Error, Result};

const FRAME_TIMEOUT: Duration = Duration::from_millis(8);
const SOURCE_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Start,
    Pause,
    Resume,
    Stop,
    Discard,
}

#[derive(Debug)]
pub enum Event {
    Prepared {
        cursor_embedded: bool,
        zoom_trigger: ZoomTrigger,
    },
    Progress {
        seconds: f64,
    },
    Finished {
        path: PathBuf,
        recording: Recording,
    },
    Discarded,
    Paused,
    Finalizing,
    Failed(String),
}

pub struct Recorder {
    commands: Sender<Command>,
    pub events: Receiver<Event>,
    worker: Option<JoinHandle<()>>,
}

impl Recorder {
    pub fn prepare(source: Source, path: PathBuf, fps: u32) -> Self {
        let (commands, receiver) = mpsc::channel();
        let (sender, events) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            if let Err(error) = record(source, path, fps, receiver, &sender) {
                let _ = sender.send(Event::Failed(error.to_string()));
            }
        });
        Self {
            commands,
            events,
            worker: Some(worker),
        }
    }

    pub fn send(&self, command: Command) -> Result<()> {
        self.commands
            .send(command)
            .map_err(|error| Error::Recording(error.to_string()))
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Stop);
        if let Some(worker) = self.worker.take() {
            // Portal selection can remain pending until the user answers it.
            if worker.is_finished() {
                let _ = worker.join();
            }
        }
    }
}

fn record(
    source: Source,
    path: PathBuf,
    fps: u32,
    commands: Receiver<Command>,
    sender: &Sender<Event>,
) -> Result<()> {
    if ![30, 60].contains(&fps) {
        return Err(Error::Recording("Unsupported frame rate".into()));
    }
    estela_media_ffmpeg::encode::available()?;
    let session = crate::platform::prepare(&source, fps)?;
    record_session(session, path, fps, commands, sender)
}

fn record_session(
    mut session: Box<dyn Session>,
    path: PathBuf,
    fps: u32,
    commands: Receiver<Command>,
    sender: &Sender<Event>,
) -> Result<()> {
    first_frame(session.as_mut())?;
    let embedded = session.cursor_embedded();
    let trigger = if embedded {
        ZoomTrigger::Unavailable
    } else if session.click_support() == estela_capture::input::InputSupport::Supported {
        ZoomTrigger::Clicks
    } else {
        ZoomTrigger::CursorDwell
    };
    sender
        .send(Event::Prepared {
            cursor_embedded: embedded,
            zoom_trigger: trigger,
        })
        .map_err(|error| Error::Recording(error.to_string()))?;
    loop {
        match commands.recv_timeout(FRAME_TIMEOUT) {
            Ok(Command::Start) => break,
            Ok(Command::Stop | Command::Discard) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                let _ = sender.send(Event::Discarded);
                return Ok(());
            }
            _ => {
                let _ = session.next_frame(FRAME_TIMEOUT)?;
            }
        }
    }
    let first = first_frame(session.as_mut())?;
    let size = (first.width, first.height);
    std::fs::create_dir_all(
        path.parent()
            .ok_or_else(|| Error::Recording("Missing destination directory".into()))?,
    )?;
    let partial = path.with_extension("partial.estela");
    let mut encoder = Encoder::start(&partial, size, fps)?;
    let mut timeline = Timeline::new(first, fps, embedded, trigger);
    let result = capture_loop(
        session.as_mut(),
        &mut encoder,
        &mut timeline,
        &commands,
        sender,
    );
    drop(session);
    let _ = sender.send(Event::Finalizing);
    let discard = result?;
    encoder.finish()?;
    if discard || timeline.frames == 0 {
        std::fs::remove_file(&partial)?;
        let _ = sender.send(Event::Discarded);
        return Ok(());
    }
    let mut recording = timeline.recording(
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("Recording"),
    );
    recording.zooms = match trigger {
        ZoomTrigger::Clicks => {
            estela_motion::autozoom::generate(&recording.events, size, recording.duration)
        }
        ZoomTrigger::CursorDwell => estela_motion::autozoom::generate_from_dwell(
            &recording.events,
            size,
            recording.duration,
        ),
        ZoomTrigger::Unavailable => Vec::new(),
    };
    estela_format::writer::finalize(&partial, &recording)?;
    if path.exists() {
        return Err(Error::Recording(
            "Destination already exists; recording retained in partial file".into(),
        ));
    }
    std::fs::rename(&partial, &path)?;
    let _ = sender.send(Event::Finished { path, recording });
    Ok(())
}

fn first_frame(session: &mut dyn Session) -> Result<Frame> {
    let start = Instant::now();
    while start.elapsed() < SOURCE_TIMEOUT {
        if let Some(frame) = session.next_frame(Duration::from_millis(100))? {
            if frame.width > 0
                && frame.height > 0
                && frame.rgba.len() == frame.width as usize * frame.height as usize * 4
            {
                return Ok(frame);
            }
            return Err(Error::Recording("Invalid capture frame".into()));
        }
    }
    Err(Error::Recording(
        "The selected source did not deliver frames".into(),
    ))
}

fn capture_loop(
    session: &mut dyn Session,
    encoder: &mut Encoder,
    timeline: &mut Timeline,
    commands: &Receiver<Command>,
    sender: &Sender<Event>,
) -> Result<bool> {
    let mut next_frame = Instant::now();
    let interval = Duration::from_secs_f64(1.0 / f64::from(timeline.fps));
    let mut paused = false;
    loop {
        match commands.try_recv() {
            Ok(Command::Stop) | Err(mpsc::TryRecvError::Disconnected) => return Ok(false),
            Ok(Command::Discard) => return Ok(true),
            Ok(Command::Pause) => {
                paused = true;
                let _ = sender.send(Event::Paused);
            }
            Ok(Command::Resume) => {
                paused = false;
                next_frame = Instant::now();
            }
            _ => {}
        }
        if let Some(frame) = session.next_frame(FRAME_TIMEOUT)? {
            if (frame.width, frame.height) != (timeline.frame.width, timeline.frame.height) {
                return Err(Error::Recording(
                    "Source dimensions changed; original fragments retained".into(),
                ));
            }
            timeline.frame = frame;
        }
        if paused {
            std::thread::sleep(FRAME_TIMEOUT);
            continue;
        }
        let now = Instant::now();
        if now < next_frame {
            std::thread::sleep((next_frame - now).min(FRAME_TIMEOUT));
            continue;
        }
        let pointer = session.pointer()?;
        let due =
            ((now - next_frame).as_nanos() / interval.as_nanos() + 1).min(u128::from(timeline.fps));
        for _ in 0..due {
            timeline.pointer(pointer);
            encoder.write(&timeline.frame.rgba)?;
            timeline.frames += 1;
            next_frame += interval;
            if timeline.frames.is_multiple_of(u64::from(timeline.fps) / 4) {
                let _ = sender.send(Event::Progress {
                    seconds: timeline.frames as f64 / f64::from(timeline.fps),
                });
            }
        }
    }
}

struct Timeline {
    frame: Frame,
    fps: u32,
    frames: u64,
    embedded: bool,
    trigger: ZoomTrigger,
    events: Vec<estela_model::Event>,
    previous: Pointer,
}

impl Timeline {
    fn new(frame: Frame, fps: u32, embedded: bool, trigger: ZoomTrigger) -> Self {
        Self {
            frame,
            fps,
            frames: 0,
            embedded,
            trigger,
            events: Vec::new(),
            previous: Pointer::default(),
        }
    }

    fn pointer(&mut self, pointer: Pointer) {
        if !pointer.available
            || pointer.x < 0
            || pointer.y < 0
            || pointer.x >= self.frame.width as i32
            || pointer.y >= self.frame.height as i32
        {
            return;
        }
        let time = recorded_time(self.frames, self.fps);
        estela_capture::record_pointer(&mut self.events, &self.previous, &pointer, time);
        self.previous = pointer;
    }

    fn recording(&self, title: &str) -> Recording {
        Recording {
            version: 1,
            title: title.into(),
            width: self.frame.width,
            height: self.frame.height,
            fps: self.fps,
            frames: self.frames,
            duration: recorded_time(self.frames, self.fps),
            cursor_embedded: self.embedded,
            zoom_trigger: self.trigger,
            events: self.events.clone(),
            zooms: Vec::new(),
        }
    }
}

fn recorded_time(frames: u64, fps: u32) -> estela_model::Time {
    estela_model::Time(
        (u128::from(frames) * 1_000_000_000 / u128::from(fps)).min(i64::MAX as u128) as i64,
    )
}

#[cfg(test)]
#[path = "recording_tests.rs"]
mod tests;
