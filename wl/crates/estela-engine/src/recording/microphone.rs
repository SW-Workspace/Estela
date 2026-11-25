use std::{
    fs::{self, File},
    io::{Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use estela_capture::microphone::{Input, Packet, SAMPLE_BYTES, SAMPLE_RATE};

use crate::{Error, Result};

const START_TIMEOUT: Duration = Duration::from_secs(4);
const TAIL_TIMEOUT: Duration = Duration::from_millis(60);
const PACKET_QUEUE_CAPACITY: usize = 128;

pub(super) struct Microphone {
    packets: Receiver<Packet>,
    worker: Worker,
    pcm: File,
    workspace: Workspace,
    origin: Instant,
    base: u64,
    written: u64,
}

impl Microphone {
    pub(super) fn open(input: Input, partial: &Path) -> Result<Option<Self>> {
        let target = match input {
            Input::Muted => return Ok(None),
            Input::Default => None,
            Input::Device(id) => Some(id),
        };
        let workspace = Workspace::create(partial)?;
        let pcm = File::create_new(workspace.path.join("microphone.pcm"))?;
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let (send, packets) = mpsc::sync_channel(PACKET_QUEUE_CAPACITY);
        let join = thread::spawn(move || {
            crate::platform::microphone::capture(target.as_deref(), send, &worker_stop)
                .map_err(|error| Error::Recording(error.to_string()))
        });
        let mut worker = Worker {
            stop,
            join: Some(join),
        };
        match packets.recv_timeout(START_TIMEOUT) {
            Ok(_) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err(worker.failure()),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                return Err(Error::Recording(
                    "Microphone did not deliver samples".into(),
                ));
            }
        }
        Ok(Some(Self {
            packets,
            worker,
            pcm,
            workspace,
            origin: Instant::now(),
            base: 0,
            written: 0,
        }))
    }

    pub(super) fn stop(&mut self) {
        self.worker.stop();
    }

    pub(super) fn resume(&mut self, origin: Instant) {
        self.origin = origin;
        self.base = self.written;
    }

    pub(super) fn drain(&mut self, paused: bool) -> Result<()> {
        while let Ok(packet) = self.packets.try_recv() {
            if !paused {
                self.write(packet, u64::MAX)?;
            }
        }
        self.worker.check()
    }

    pub(super) fn end_segment(&mut self, frames: u64, fps: u32) -> Result<()> {
        let end = frames * u64::from(SAMPLE_RATE / fps);
        let deadline = Instant::now() + TAIL_TIMEOUT;
        while self.written < end {
            match self
                .packets
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            {
                Ok(packet) => self.write(packet, end)?,
                Err(_) => break,
            }
        }
        self.worker.check()?;
        self.seek(end)?;
        self.pcm.set_len(end * SAMPLE_BYTES as u64)?;
        self.written = end;
        Ok(())
    }

    fn write(&mut self, packet: Packet, end: u64) -> Result<()> {
        let offset = |duration: Duration| {
            (duration.as_nanos() * u128::from(SAMPLE_RATE)).div_ceil(1_000_000_000) as u64
        };
        let (start, before_origin) = if packet.start >= self.origin {
            (self.base + offset(packet.start - self.origin), 0)
        } else {
            (self.base, offset(self.origin - packet.start))
        };
        let skip = before_origin + self.written.saturating_sub(start);
        let write_at = start.max(self.written);
        let count = (packet.pcm.len() / SAMPLE_BYTES) as u64;
        let remaining = count.saturating_sub(skip).min(end.saturating_sub(write_at));
        if remaining == 0 {
            return Ok(());
        }
        if write_at > self.written {
            self.seek(write_at)?;
        }
        let first = skip as usize * SAMPLE_BYTES;
        let last = first + remaining as usize * SAMPLE_BYTES;
        self.pcm.write_all(&packet.pcm[first..last])?;
        self.written = write_at + remaining;
        Ok(())
    }

    fn seek(&mut self, sample: u64) -> Result<()> {
        self.pcm
            .seek(SeekFrom::Start(sample * SAMPLE_BYTES as u64))?;
        Ok(())
    }

    pub(super) fn mux(self, partial: &Path) -> Result<()> {
        let Self {
            pcm, mut workspace, ..
        } = self;
        drop(pcm);
        let output = workspace.path.join("recording.mp4");
        let raw = workspace.path.join("microphone.pcm");
        let result = estela_media_ffmpeg::mux::microphone(partial, &raw, &output, SAMPLE_RATE)
            .map_err(Error::from)
            .and_then(|()| Ok(fs::rename(output, partial)?));
        if let Err(error) = result {
            workspace.keep = true;
            return Err(Error::Recording(format!(
                "{error}. Microphone audio retained at {}",
                raw.display()
            )));
        }
        Ok(())
    }
}

struct Worker {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<Result<()>>>,
}

impl Worker {
    fn check(&mut self) -> Result<()> {
        if self
            .join
            .as_ref()
            .is_some_and(|handle| handle.is_finished())
        {
            return Err(self.failure());
        }
        Ok(())
    }

    fn failure(&mut self) -> Error {
        match self.join.take().map(JoinHandle::join) {
            Some(Ok(Err(error))) => error,
            Some(Err(_)) => Error::Recording("Microphone worker panicked".into()),
            _ => Error::Recording("Microphone stopped delivering audio".into()),
        }
    }

    fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop();
    }
}

struct Workspace {
    path: PathBuf,
    keep: bool,
}

impl Workspace {
    fn create(partial: &Path) -> Result<Self> {
        let path = partial.with_extension("audio");
        fs::create_dir(&path)?;
        Ok(Self { path, keep: false })
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        if !self.keep {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
