use estela_model::{Event, Time, Zoom};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub version: u32,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub frames: u64,
    pub duration: Time,
    pub cursor_embedded: bool,
    pub zoom_trigger: ZoomTrigger,
    pub events: Vec<Event>,
    pub zooms: Vec<Zoom>,
}

impl Recording {
    pub fn validate(&self) -> crate::Result<()> {
        if self.version != 1
            || self.width == 0
            || self.height == 0
            || self.width > 16384
            || self.height > 16384
            || ![30, 60].contains(&self.fps)
            || self.frames == 0
            || self.duration.0 <= 0
            || self
                .events
                .iter()
                .any(|event| event.t.0 < 0 || event.t > self.duration)
            || self.events.windows(2).any(|pair| pair[0].t > pair[1].t)
        {
            return Err(crate::Error::Invalid("Invalid recording metadata".into()));
        }
        if self.zooms.iter().any(|zoom| {
            zoom.range.start.0 < 0
                || zoom.range.end > self.duration
                || zoom.range.start >= zoom.range.end
                || !zoom.level.is_finite()
                || !(1.0..=3.0).contains(&zoom.level)
        }) {
            return Err(crate::Error::Invalid("Invalid zoom metadata".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoomTrigger {
    Clicks,
    CursorDwell,
    Unavailable,
}
