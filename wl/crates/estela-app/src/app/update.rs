use super::*;

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => return self.tick(),
            Message::OpenedWindow(id) => {
                if id != self.window {
                    return Task::none();
                }
                let tasks = window::scale_factor(id).map(move |scale| Message::Scale(id, scale));
                if let Some(path) = std::env::args_os().nth(1).map(PathBuf::from) {
                    return Task::batch([tasks, load(path)]);
                }
                return tasks;
            }
            Message::Configured(Err(error)) => self.error = Some(error),
            Message::Configured(Ok(excluded)) => self.excluded = excluded,
            Message::Scale(id, scale) if id == self.window => {
                self.scale = scale;
                return self.configure_input();
            }
            Message::Moved(id, point) if id == self.window => {
                self.preferences.position = Some((point.x, point.y));
                self.save_preferences();
            }
            Message::Sources(result) => match result {
                Ok(sources) => {
                    self.sources = sources;
                    self.select_first();
                }
                Err(error) => {
                    self.error = Some(error);
                    self.popover = Some(Popover::Source);
                    return self.configure_input();
                }
            },
            Message::Refresh if self.state == State::Ready => {
                self.discard_selection();
                return refresh();
            }
            Message::TogglePopover(popover) => {
                self.popover = if self.popover == Some(popover) {
                    None
                } else {
                    Some(popover)
                };
                return self.configure_input();
            }
            Message::DismissPopover => {
                self.popover = None;
                return self.configure_input();
            }
            Message::RetryShortcuts => {
                if !self.shortcuts_ready {
                    self.shortcuts_failed = false;
                    self.shortcuts = api::shortcuts([
                        self.strings.get("stop-shortcut").into(),
                        self.strings.get("pause-shortcut").into(),
                    ]);
                }
            }
            Message::Kind(kind) if self.state == State::Ready => {
                self.discard_selection();
                self.kind = kind;
                self.select_first();
                self.popover = Some(Popover::Source);
                return self.configure_input();
            }
            Message::ChooseSource if self.state == State::Ready => {
                self.discard_selection();
                self.selecting = true;
                return self.prepare();
            }
            Message::Source(source) if self.state == State::Ready => {
                self.discard_selection();
                let picker = source.selection == api::SourceSelection::SystemPicker;
                self.selected = Some(source);
                if picker {
                    self.selecting = true;
                    return self.prepare();
                }
                self.popover = None;
                return self.configure_input();
            }
            Message::Fps(fps) if self.state == State::Ready => {
                self.discard_selection();
                self.preferences.fps = fps;
                self.save_preferences();
            }
            Message::Countdown(enabled) if self.state == State::Ready => {
                self.preferences.countdown = enabled;
                self.save_preferences();
            }
            Message::Record => return self.prepare(),
            Message::Start => {
                if self.state == State::Starting {
                    self.send(if self.resuming {
                        recording::Command::Resume
                    } else {
                        recording::Command::Start
                    });
                    self.resuming = false;
                    self.state = State::Recording;
                }
            }
            Message::Pause => return self.pause(),
            Message::Stop => return self.stop(false),
            Message::Restart => {
                self.restarting = true;
                return self.stop(true);
            }
            Message::Cancel => {
                if self.popover.is_some()
                    && matches!(self.state, State::Ready | State::Recording | State::Paused)
                {
                    self.popover = None;
                    return self.configure_input();
                }
                if self.fullscreen {
                    self.popover = None;
                    self.fullscreen = false;
                    return window::set_mode(
                        self.preview_window.unwrap_or(self.window),
                        window::Mode::Windowed,
                    );
                }
                if matches!(
                    self.state,
                    State::Countdown | State::Preparing | State::Starting
                ) {
                    return self.stop(true);
                }
            }
            Message::Drag => return window::drag(self.preview_window.unwrap_or(self.window)),
            Message::Close => {
                if self.recorder.is_some()
                    && !matches!(
                        self.state,
                        State::Ready | State::Preparing | State::Countdown
                    )
                {
                    self.closing = true;
                    return self.stop(false);
                }
                self.recorder.take();
                self.player.take();
                return self.exit();
            }
            Message::Dismiss => self.error = None,
            Message::New => {
                self.fullscreen = false;
                self.popover = None;
                self.player.take();
                self.state = State::Ready;
                self.image = None;
                self.recording = None;
                return self.update_preview_window();
            }
            Message::Open => {
                let title = self.strings.get("open").to_owned();
                let description = self.strings.get("recording-file").to_owned();
                return Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .set_title(title)
                            .add_filter(description, &["estela"])
                            .pick_file()
                            .await
                            .map(|file| file.path().to_owned())
                    },
                    Message::Picked,
                );
            }
            Message::Picked(Some(path)) => {
                self.state = State::Loading;
                return Task::batch([load(path), self.configure_input()]);
            }
            Message::Loaded(path, result) => match result {
                Ok(recording) => {
                    self.fullscreen = false;
                    self.seek_target = None;
                    self.player = Some(Player::open(path.clone(), recording.clone()));
                    self.path = Some(path);
                    self.recording = Some(recording);
                    self.image = None;
                    self.position = 0.0;
                    self.playing = false;
                    self.autozoom = true;
                    self.state = State::Playback;
                    return self.update_preview_window();
                }
                Err(error) => {
                    self.error = Some(error);
                    self.state = if self.preview_window.is_some() && self.recording.is_some() {
                        State::Playback
                    } else {
                        self.popover = Some(Popover::Source);
                        State::Ready
                    };
                    return self.configure_input();
                }
            },
            Message::Play => {
                if let Some(player) = &self.player {
                    self.playing = !self.playing;
                    if self.playing && self.position >= self.duration() - 0.05 {
                        player.send(preview::Command::Seek(0.0));
                    }
                    player.send(if self.playing {
                        preview::Command::Play
                    } else {
                        preview::Command::Pause
                    });
                }
            }
            Message::Seek(seconds) => {
                self.seek_target = Some(seconds);
                self.position = seconds;
                self.playing = false;
                if let Some(player) = &self.player {
                    player.send(preview::Command::Pause);
                }
            }
            Message::SeekCommit => {
                self.position = self.seek_target.take().unwrap_or(self.position);
                if let Some(player) = &self.player {
                    player.send(preview::Command::Seek(self.position));
                }
            }
            Message::Step(direction) => {
                self.playing = false;
                self.position = (self.position
                    + f64::from(direction)
                        / f64::from(
                            self.recording
                                .as_ref()
                                .map_or(60, |recording| recording.fps),
                        ))
                .clamp(0.0, self.duration());
                if let Some(player) = &self.player {
                    player.send(preview::Command::Pause);
                    player.send(preview::Command::Seek(self.position));
                }
            }
            Message::AutoZoom(enabled) => {
                self.autozoom = enabled;
                if let Some(player) = &self.player {
                    player.send(preview::Command::AutoZoom(enabled));
                }
            }
            Message::Fullscreen => {
                self.fullscreen = !self.fullscreen;
                return window::set_mode(
                    self.preview_window.unwrap_or(self.window),
                    if self.fullscreen {
                        window::Mode::Fullscreen
                    } else {
                        window::Mode::Windowed
                    },
                );
            }
            _ => {}
        }
        Task::none()
    }
}
