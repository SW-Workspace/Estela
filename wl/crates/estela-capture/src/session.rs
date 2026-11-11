use std::time::Duration;

use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Display,
    Window,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceSelection {
    Direct,
    SystemPicker,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub id: u64,
    pub name: String,
    pub kind: SourceKind,
    pub selection: SourceSelection,
}

impl std::fmt::Display for Source {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.name)
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Pointer {
    pub x: i32,
    pub y: i32,
    pub buttons: [bool; 3],
    pub available: bool,
}

pub trait Session: Send {
    fn next_frame(&mut self, timeout: Duration) -> Result<Option<Frame>>;
    fn pointer(&self) -> Result<Pointer>;
    fn cursor_embedded(&self) -> bool;
    fn click_support(&self) -> crate::input::InputSupport {
        crate::input::InputSupport::Supported
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shortcut {
    Ready,
    Unavailable,
    Record,
    Pause,
}
