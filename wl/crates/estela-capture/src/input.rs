use estela_model::Event;

use crate::Result;

#[derive(Debug, Clone, Copy)]
pub struct InputCaps {
    pub position: InputSupport,
    pub clicks: InputSupport,
    pub cursor_shapes: InputSupport,
}
pub trait InputSource: Send {
    fn capabilities(&self) -> InputCaps;
    fn start(&mut self, tx: crossbeam_channel::Sender<Event>) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSupport {
    Unsupported,
    Supported,
}
