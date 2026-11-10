use serde::{Deserialize, Serialize};

use crate::time::Time;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub t: Time,
    pub kind: EventKind,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    Move { x: i32, y: i32 },
    Down { button: Button },
    Up { button: Button },
    CursorShape { id: CursorId },
    Dropped,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Button {
    Left,
    Right,
    Middle,
    Other(u16),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CursorId(pub u64);
