use serde::{Deserialize, Serialize};

use estela_model::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    AddZoom { range: TimeRange, mode: ZoomMode },
    MoveZoom { id: ElementId, range: TimeRange },
    RemoveZoom { id: ElementId },
    Trim { range: TimeRange },
    SetCursorStyle(CursorStyle),
    SetBackground(Background),
}
