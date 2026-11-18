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

pub fn record_pointer(
    events: &mut Vec<Event>,
    previous: &super::session::Pointer,
    pointer: &super::session::Pointer,
    time: estela_model::Time,
) {
    use estela_model::{Button, EventKind};
    events.push(Event {
        t: time,
        kind: EventKind::Move {
            x: pointer.x,
            y: pointer.y,
        },
    });
    for (index, button) in [Button::Left, Button::Right, Button::Middle]
        .into_iter()
        .enumerate()
    {
        if pointer.buttons[index] != previous.buttons[index] {
            let kind = if pointer.buttons[index] {
                EventKind::Down { button }
            } else {
                EventKind::Up { button }
            };
            events.push(Event { t: time, kind });
        }
    }
}
