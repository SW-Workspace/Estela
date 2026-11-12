mod flow;
mod update;

use std::{
    path::PathBuf,
    sync::mpsc::Receiver,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use iced::{window, Element, Size, Subscription, Task};
use serde::{Deserialize, Serialize};

use estela_engine::{
    api::{self, Recording, Shortcut, Source, SourceKind, ZoomTrigger},
    preview::{self, Player},
    recording::{self, Recorder},
};

use crate::{localization::Strings, message::Message};

const BAR_SIZE: Size = Size::new(600.0, 440.0);
const SETUP_SIZE: Size = Size::new(600.0, 440.0);
const PREVIEW_SIZE: Size = Size::new(1120.0, 780.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
    Ready,
    Preparing,
    Countdown,
    Starting,
    Recording,
    Paused,
    Saving,
    Loading,
    Playback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Popover {
    Source,
    Microphone,
    Menu,
    Settings,
    Shortcuts,
}

impl Popover {
    pub(crate) fn panel_size(self) -> (i32, i32) {
        match self {
            Self::Source => (360, 340),
            Self::Microphone => (300, 200),
            Self::Menu => (260, 200),
            Self::Settings => (360, 220),
            Self::Shortcuts => (480, 340),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Preferences {
    pub countdown: bool,
    pub fps: u32,
    pub position: Option<(f32, f32)>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            countdown: true,
            fps: 60,
            position: None,
        }
    }
}

pub struct App {
    pub(crate) strings: Strings,
    pub(crate) state: State,
    pub(crate) popover: Option<Popover>,
    pub(crate) preferences: Preferences,
    pub(crate) sources: Vec<Source>,
    pub(crate) selected: Option<Source>,
    pub(crate) kind: SourceKind,
    pub(crate) error: Option<String>,
    pub(crate) elapsed: f64,
    pub(crate) countdown: u64,
    pub(crate) recording: Option<Recording>,
    pub(crate) path: Option<PathBuf>,
    pub(crate) image: Option<iced::widget::image::Handle>,
    pub(crate) position: f64,
    seek_target: Option<f64>,
    pub(crate) playing: bool,
    pub(crate) autozoom: bool,
    pub(crate) shortcuts_ready: bool,
    pub(crate) shortcuts_failed: bool,
    pub(crate) embedded: bool,
    pub(crate) zoom_trigger: ZoomTrigger,
    pub(crate) excluded: bool,
    window: window::Id,
    preview_window: Option<window::Id>,
    scale: f32,
    selecting: bool,
    prepared: bool,
    recorder: Option<Recorder>,
    player: Option<Player>,
    shortcuts: Receiver<Shortcut>,
    countdown_start: Option<Instant>,
    closing: bool,
    restarting: bool,
    fullscreen: bool,
    resuming: bool,
}

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        let strings = Strings::default();
        let shortcuts = api::shortcuts([
            strings.get("stop-shortcut").into(),
            strings.get("pause-shortcut").into(),
        ]);
        let mut preferences: Preferences = std::fs::read(preferences_path())
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        if ![30, 60].contains(&preferences.fps) {
            preferences.fps = 60;
        }
        preferences.position = preferences
            .position
            .filter(|(x, y)| x.is_finite() && y.is_finite());
        let position = preferences
            .position
            .map(|(x, y)| window::Position::Specific(iced::Point::new(x, y)))
            .unwrap_or(window::Position::Centered);
        let (id, open) = window::open(window::Settings {
            size: BAR_SIZE,
            position,
            decorations: false,
            transparent: true,
            resizable: false,
            level: window::Level::AlwaysOnTop,
            exit_on_close_request: false,
            ..Default::default()
        });
        let app = Self {
            strings,
            state: State::Ready,
            popover: None,
            preferences,
            sources: Vec::new(),
            selected: None,
            kind: SourceKind::Display,
            error: None,
            elapsed: 0.0,
            countdown: 3,
            recording: None,
            path: None,
            image: None,
            position: 0.0,
            seek_target: None,
            playing: false,
            autozoom: true,
            shortcuts_ready: false,
            shortcuts_failed: false,
            embedded: false,
            zoom_trigger: ZoomTrigger::Unavailable,
            excluded: false,
            window: id,
            preview_window: None,
            scale: 1.0,
            selecting: false,
            prepared: false,
            recorder: None,
            player: None,
            shortcuts,
            countdown_start: None,
            closing: false,
            restarting: false,
            fullscreen: false,
            resuming: false,
        };
        (
            app,
            Task::batch([open.map(Message::OpenedWindow), refresh()]),
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick),
            window::events().filter_map(|(id, event)| match event {
                window::Event::CloseRequested => Some(Message::Close),
                window::Event::Moved(point) => Some(Message::Moved(id, point)),
                window::Event::Rescaled(scale) => Some(Message::Scale(id, scale)),
                _ => None,
            }),
            iced::keyboard::listen().filter_map(|event| match event {
                iced::keyboard::Event::KeyPressed {
                    key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                    ..
                } => Some(Message::Cancel),
                _ => None,
            }),
        ])
    }

    pub fn view(&self, window: window::Id) -> Element<'_, Message> {
        if self.preview_window == Some(window) {
            return crate::widgets::preview::view(self);
        }
        if self.controls_hidden() {
            return iced::widget::space().into();
        }
        crate::views::recorder::view(self)
    }

    fn select_first(&mut self) {
        self.selected = self
            .sources
            .iter()
            .find(|source| source.kind == self.kind)
            .cloned();
    }

    pub(crate) fn countdown_fraction(&self) -> f32 {
        self.countdown_start.map_or(0.0, |start| {
            (1.0 - start.elapsed().as_secs_f32() / 3.0).clamp(0.0, 1.0)
        })
    }

    pub(crate) fn duration(&self) -> f64 {
        self.recording
            .as_ref()
            .map_or(0.0, |recording| recording.duration.0 as f64 / 1e9)
    }

    fn update_preview_window(&mut self) -> Task<Message> {
        if self.state != State::Playback {
            let close = self
                .preview_window
                .take()
                .map(window::close)
                .unwrap_or_else(Task::none);
            return Task::batch([close, self.configure_input()]);
        }
        if self.preview_window.is_some() {
            return self.configure_input();
        }
        let (id, open) = window::open(window::Settings {
            size: PREVIEW_SIZE,
            min_size: Some(SETUP_SIZE),
            position: window::Position::Centered,
            decorations: false,
            exit_on_close_request: false,
            ..Default::default()
        });
        self.preview_window = Some(id);
        Task::batch([open.map(Message::OpenedWindow), self.configure_input()])
    }

    pub(crate) fn controls_hidden(&self) -> bool {
        matches!(
            self.state,
            State::Playback | State::Loading | State::Preparing
        ) || (!self.excluded
            && matches!(self.state, State::Starting | State::Recording)
            && self
                .selected
                .as_ref()
                .is_some_and(|source| source.kind == SourceKind::Display))
    }

    fn configure_input(&self) -> Task<Message> {
        let input = self.input_region();
        let scale = self.scale;
        window::run(self.window, move |window| {
            let window_handle = window.window_handle().map_err(|error| error.to_string())?;
            let display = window.display_handle().map_err(|error| error.to_string())?;
            api::configure_window(window_handle, display, &input, scale)
                .map_err(|error| error.to_string())
        })
        .map(Message::Configured)
    }

    fn input_region(&self) -> api::WindowInput {
        if self.controls_hidden() {
            return api::WindowInput::Areas(Vec::new());
        }
        let card = api::InputArea {
            x: 140,
            y: 30,
            width: 320,
            height: 380,
            radius: 16,
        };
        if !matches!(self.state, State::Ready | State::Recording | State::Paused) {
            return api::WindowInput::Areas(vec![card]);
        }
        let mut areas = vec![api::InputArea {
            x: 16,
            y: 16,
            width: 568,
            height: 56,
            radius: 28,
        }];
        if let Some(popover) = self.popover {
            let (width, height) = popover.panel_size();
            areas.push(api::InputArea {
                x: (600 - width) / 2,
                y: 84,
                width: width as u32,
                height: height as u32,
                radius: 14,
            });
        }
        api::WindowInput::Areas(areas)
    }

    fn exit(&self) -> Task<Message> {
        window::run(self.window, |window| {
            if let Ok(display) = window.display_handle() {
                api::release_window(display);
            }
        })
        .then(|()| iced::exit())
    }

    fn save_preferences(&self) {
        let path = preferences_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(bytes) = serde_json::to_vec(&self.preferences) {
            let _ = std::fs::write(path, bytes);
        }
    }
}

fn refresh() -> Task<Message> {
    Task::perform(
        async { api::sources().map_err(|error| error.to_string()) },
        Message::Sources,
    )
}
fn load(path: PathBuf) -> Task<Message> {
    Task::perform(
        async move {
            let result = api::open(&path).map_err(|error| error.to_string());
            (path, result)
        },
        |(path, result)| Message::Loaded(path, result),
    )
}
fn preferences_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("Estela")
        .join("recorder.json")
}
