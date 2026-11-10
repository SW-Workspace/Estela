use estela_model::*;

use crate::{command::Command, Error, Result};

const DEFAULT_ZOOM_LEVEL: f64 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryOutcome {
    Applied,
    Unavailable,
}

pub struct Timeline {
    edit: Edit,
    past: Vec<Edit>,
    future: Vec<Edit>,
}
impl Timeline {
    pub fn new(edit: Edit) -> Self {
        Self {
            edit,
            past: Vec::new(),
            future: Vec::new(),
        }
    }
    pub fn edit(&self) -> &Edit {
        &self.edit
    }
    pub fn apply(&mut self, command: Command) -> Result<()> {
        let mut next = self.edit.clone();
        match command {
            Command::AddZoom { range, mode } => next.zooms.push(Zoom {
                id: ElementId(ulid_new()),
                range,
                mode,
                level: DEFAULT_ZOOM_LEVEL,
            }),
            Command::RemoveZoom { id } => next.zooms.retain(|zoom| zoom.id != id),
            _ => return Err(Error::NotImplemented),
        }
        self.past.push(std::mem::replace(&mut self.edit, next));
        self.future.clear();
        Ok(())
    }
    pub fn undo(&mut self) -> HistoryOutcome {
        if let Some(previous) = self.past.pop() {
            self.future
                .push(std::mem::replace(&mut self.edit, previous));
            HistoryOutcome::Applied
        } else {
            HistoryOutcome::Unavailable
        }
    }
    pub fn redo(&mut self) -> HistoryOutcome {
        if let Some(next) = self.future.pop() {
            self.past.push(std::mem::replace(&mut self.edit, next));
            HistoryOutcome::Applied
        } else {
            HistoryOutcome::Unavailable
        }
    }
}
fn ulid_new() -> ulid::Ulid {
    ulid::Ulid::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_history_restores_exact_state_and_invalidates_redo() {
        let initial = Edit {
            clips: vec![],
            zooms: vec![],
            cursor: CursorStyle {
                visible: Visibility::Visible,
                scale: 1.0,
            },
            background: Background::Solid([0.0, 0.0, 0.0, 1.0]),
            export: ExportSettings {
                size: Size {
                    width: 1920.0,
                    height: 1080.0,
                },
                fps: 60.0,
                bitrate_kbps: 8000,
            },
            extra: Default::default(),
        };
        let mut timeline = Timeline::new(initial.clone());
        let add = || Command::AddZoom {
            range: TimeRange {
                start: Time(0),
                end: Time(1_000_000_000),
            },
            mode: ZoomMode::Auto,
        };
        timeline.apply(add()).unwrap();
        let added = timeline.edit().clone();
        timeline
            .apply(Command::RemoveZoom {
                id: added.zooms[0].id,
            })
            .unwrap();
        assert_eq!(timeline.edit(), &initial);
        assert_eq!(timeline.undo(), HistoryOutcome::Applied);
        assert_eq!(timeline.edit(), &added);
        assert_eq!(timeline.undo(), HistoryOutcome::Applied);
        assert_eq!(timeline.edit(), &initial);
        assert_eq!(timeline.undo(), HistoryOutcome::Unavailable);
        assert_eq!(timeline.redo(), HistoryOutcome::Applied);
        assert_eq!(timeline.edit(), &added);
        assert_eq!(timeline.undo(), HistoryOutcome::Applied);
        timeline.apply(add()).unwrap();
        assert_eq!(timeline.redo(), HistoryOutcome::Unavailable);
    }
}
