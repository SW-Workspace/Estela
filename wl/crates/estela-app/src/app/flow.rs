use super::*;

impl App {
    pub(super) fn prepare(&mut self) -> Task<Message> {
        if self.state != State::Ready {
            return Task::none();
        }
        let Some(source) = self.selected.clone() else {
            return Task::none();
        };
        if !self.selecting
            && source.kind == SourceKind::Display
            && !self.excluded
            && !self.shortcuts_ready
        {
            self.error = Some(self.strings.get("shortcuts-failed").into());
            self.popover = Some(Popover::Shortcuts);
            return self.configure_input();
        }
        self.popover = None;
        self.error = None;
        if self.prepared {
            self.begin_countdown();
            return self.configure_input();
        }
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let directory = dirs::video_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(std::env::temp_dir)
            .join("Estela");
        let path = directory.join(format!(
            "{}-{timestamp}.estela",
            self.strings.get("recording")
        ));
        self.recorder = Some(Recorder::prepare(source, path, self.preferences.fps));
        self.state = State::Preparing;
        self.elapsed = 0.0;
        self.configure_input()
    }

    fn begin_countdown(&mut self) {
        self.prepared = false;
        self.countdown_start = Some(Instant::now());
        self.state = State::Countdown;
        self.countdown = 3;
    }

    pub(super) fn discard_selection(&mut self) {
        if self.prepared {
            self.recorder.take();
        }
        self.prepared = false;
        self.selecting = false;
    }

    pub(super) fn pause(&mut self) -> Task<Message> {
        if self.state == State::Recording {
            self.send(recording::Command::Pause);
            return Task::none();
        }
        if self.state == State::Paused {
            self.resuming = true;
            self.state = State::Starting;
            return Task::batch([
                self.configure_input(),
                Task::perform(
                    async {
                        std::thread::sleep(Duration::from_millis(300));
                    },
                    |()| Message::Start,
                ),
            ]);
        }
        Task::none()
    }

    pub(super) fn stop(&mut self, discard: bool) -> Task<Message> {
        if self.recorder.is_none() || self.state == State::Saving {
            return Task::none();
        }
        self.send(if discard {
            recording::Command::Discard
        } else {
            recording::Command::Stop
        });
        self.state = State::Saving;
        self.countdown_start = None;
        self.prepared = false;
        self.selecting = false;
        self.configure_input()
    }

    pub(super) fn send(&mut self, command: recording::Command) {
        if let Some(recorder) = &self.recorder {
            if let Err(error) = recorder.send(command) {
                self.error = Some(error.to_string());
            }
        }
    }

    pub(super) fn tick(&mut self) -> Task<Message> {
        let mut tasks = Vec::new();
        while let Ok(shortcut) = self.shortcuts.try_recv() {
            match shortcut {
                Shortcut::Ready => {
                    self.shortcuts_ready = true;
                    self.shortcuts_failed = false;
                }
                Shortcut::Unavailable => {
                    self.shortcuts_failed = true;
                    if self.state == State::Ready {
                        self.popover = Some(Popover::Shortcuts);
                        tasks.push(self.configure_input());
                    }
                }
                Shortcut::Record => tasks.push(if self.state == State::Ready {
                    self.prepare()
                } else {
                    self.stop(false)
                }),
                Shortcut::Pause => tasks.push(self.pause()),
            }
        }
        let events: Vec<_> = self
            .recorder
            .as_ref()
            .map(|recorder| recorder.events.try_iter().collect())
            .unwrap_or_default();
        for event in events {
            match event {
                recording::Event::Prepared {
                    cursor_embedded,
                    zoom_trigger,
                } => {
                    self.embedded = cursor_embedded;
                    self.zoom_trigger = zoom_trigger;
                    if self.state == State::Preparing {
                        if self.selecting {
                            self.selecting = false;
                            self.prepared = true;
                            self.state = State::Ready;
                        } else {
                            self.begin_countdown();
                        }
                        tasks.push(self.configure_input());
                    }
                }
                recording::Event::Paused => {
                    self.state = State::Paused;
                    tasks.push(self.configure_input());
                }
                recording::Event::Finalizing => tasks.push(self.configure_input()),
                recording::Event::Progress { seconds } => self.elapsed = seconds,
                recording::Event::Finished { path, recording } => {
                    self.recorder.take();
                    if self.closing {
                        return self.exit();
                    }
                    tasks.push(self.update(Message::Loaded(path, Ok(recording))));
                }
                recording::Event::Discarded => {
                    self.recorder.take();
                    if self.closing {
                        return self.exit();
                    }
                    self.state = State::Ready;
                    if self.restarting {
                        self.restarting = false;
                        tasks.push(self.prepare());
                    } else {
                        tasks.push(self.configure_input());
                    }
                }
                recording::Event::Failed(error) => {
                    if self.closing {
                        return self.exit();
                    }
                    self.popover = Some(Popover::Source);
                    self.error = Some(error);
                    self.recorder.take();
                    self.prepared = false;
                    self.selecting = false;
                    self.countdown_start = None;
                    self.restarting = false;
                    self.resuming = false;
                    self.state = State::Ready;
                    tasks.push(self.configure_input());
                }
            }
        }
        if let Some(start) = self.countdown_start {
            let remaining = 3_u64.saturating_sub(start.elapsed().as_secs());
            self.countdown = remaining;
            if remaining == 0 || !self.preferences.countdown {
                self.countdown_start = None;
                self.state = State::Starting;
                tasks.push(self.configure_input());
                tasks.push(Task::perform(
                    async {
                        std::thread::sleep(Duration::from_millis(300));
                    },
                    |()| Message::Start,
                ));
            }
        }
        if let Some(player) = &self.player {
            for event in player.events.try_iter() {
                match event {
                    preview::Event::Frame {
                        rgba,
                        size,
                        seconds,
                    } => {
                        self.image =
                            Some(iced::widget::image::Handle::from_rgba(size.0, size.1, rgba));
                        if self.seek_target.is_none() {
                            self.position = seconds;
                        }
                    }
                    preview::Event::End => {
                        self.playing = false;
                        self.position = self.duration();
                    }
                    preview::Event::Failed(error) => {
                        self.popover = Some(Popover::Shortcuts);
                        self.error = Some(error);
                        self.playing = false;
                    }
                }
            }
        }
        Task::batch(tasks)
    }
}
