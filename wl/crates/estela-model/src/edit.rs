use serde::{Deserialize, Serialize};

use crate::{
    geometry::{Point, Size},
    ids::ElementId,
    time::TimeRange,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edit {
    pub clips: Vec<Clip>,
    pub zooms: Vec<Zoom>,
    pub cursor: CursorStyle,
    pub background: Background,
    pub export: ExportSettings,
    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Clip {
    pub id: ElementId,
    pub range: TimeRange,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Zoom {
    pub id: ElementId,
    pub range: TimeRange,
    pub level: f64,
    pub mode: ZoomMode,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ZoomMode {
    Auto,
    Manual { center: Point },
    FollowCursor,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CursorStyle {
    pub visible: Visibility,
    pub scale: f64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Background {
    Solid([f64; 4]),
    Image(String),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportSettings {
    pub size: Size,
    pub fps: f64,
    pub bitrate_kbps: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "bool", into = "bool")]
pub enum Visibility {
    Hidden,
    Visible,
}

impl From<bool> for Visibility {
    fn from(value: bool) -> Self {
        if value {
            Self::Visible
        } else {
            Self::Hidden
        }
    }
}

impl From<Visibility> for bool {
    fn from(value: Visibility) -> Self {
        value == Visibility::Visible
    }
}
