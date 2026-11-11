use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::Duration,
};

use ashpd::desktop::{
    screencast::{CursorMode, Screencast, SelectSourcesOptions, SourceType},
    Session as PortalSession,
};

use estela_capture::{
    session::{Frame, Pointer, Session, Source, SourceKind},
    Error, Result,
};

fn backend(error: impl std::fmt::Display) -> Error {
    Error::Backend(error.to_string())
}

struct Capture {
    frames: Receiver<Result<Frame>>,
    pointer: Arc<Mutex<Pointer>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    embedded: bool,
    session: PortalSession<Screencast>,
    runtime: &'static tokio::runtime::Runtime,
}

pub fn prepare(source: &Source, fps: u32) -> Result<Box<dyn Session>> {
    let runtime = crate::portal_runtime::runtime().map_err(backend)?;
    let (session, remote, node, embedded) = runtime
        .block_on(async {
            crate::portal_runtime::register_application().await;
            let proxy = Screencast::new().await?;
            let session = proxy.create_session(Default::default()).await?;
            let kind = if source.kind == SourceKind::Display {
                SourceType::MONITOR
            } else {
                SourceType::WINDOW
            };
            let modes = proxy.available_cursor_modes().await?;
            let mode = if modes.contains(CursorMode::METADATA) {
                CursorMode::METADATA
            } else {
                CursorMode::EMBEDDED
            };
            proxy
                .select_sources(
                    &session,
                    SelectSourcesOptions::default()
                        .set_sources(kind)
                        .set_multiple(false)
                        .set_cursor_mode(mode),
                )
                .await?
                .response()?;
            let response = proxy
                .start(&session, None, Default::default())
                .await?
                .response()?;
            let stream = response
                .streams()
                .first()
                .ok_or(ashpd::Error::ParseError("No source selected"))?;
            let node = stream.pipe_wire_node_id();
            let remote = proxy
                .open_pipe_wire_remote(&session, Default::default())
                .await?;
            Ok::<_, ashpd::Error>((session, remote, node, mode == CursorMode::EMBEDDED))
        })
        .map_err(backend)?;
    let (sender, frames) = mpsc::sync_channel(2);
    let pointer = Arc::new(Mutex::new(Pointer::default()));
    let stop = Arc::new(AtomicBool::new(false));
    let (worker_pointer, worker_stop) = (pointer.clone(), stop.clone());
    let worker = std::thread::spawn(move || {
        if let Err(error) = super::pipewire::capture(
            remote,
            node,
            fps,
            sender.clone(),
            worker_pointer,
            worker_stop,
        ) {
            let _ = sender.send(Err(error));
        }
    });
    Ok(Box::new(Capture {
        frames,
        pointer,
        stop,
        worker: Some(worker),
        embedded,
        session,
        runtime,
    }))
}

impl Session for Capture {
    fn next_frame(&mut self, timeout: Duration) -> Result<Option<Frame>> {
        match self.frames.recv_timeout(timeout) {
            Ok(frame) => frame.map(Some),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(error) => Err(backend(error)),
        }
    }
    fn pointer(&self) -> Result<Pointer> {
        self.pointer.lock().map(|pointer| *pointer).map_err(backend)
    }
    fn cursor_embedded(&self) -> bool {
        self.embedded
    }
    fn click_support(&self) -> estela_capture::input::InputSupport {
        estela_capture::input::InputSupport::Unsupported
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        let _ = self.runtime.block_on(self.session.close());
    }
}
