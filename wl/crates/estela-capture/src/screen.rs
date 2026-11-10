use estela_media::frame::NativeFrame;
use estela_model::{Rect, Time};

use crate::Result;

#[derive(Debug, Clone, Copy)]
pub struct DisplayId(pub u64);
#[derive(Debug, Clone, Copy)]
pub struct WindowId(pub u64);
pub enum CaptureTarget {
    Display(DisplayId),
    Window(WindowId),
    Region { display: DisplayId, rect: Rect },
}
pub struct Display {
    pub id: DisplayId,
    pub name: String,
    pub bounds: Rect,
    pub scale: f64,
}
pub struct CapturedFrame {
    pub pts: Time,
    pub frame: NativeFrame,
    pub size: (u32, u32),
}
pub trait ScreenSource: Send {
    fn displays(&self) -> Result<Vec<Display>>;
    fn start(
        &mut self,
        target: CaptureTarget,
        tx: crossbeam_channel::Sender<CapturedFrame>,
    ) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
}
