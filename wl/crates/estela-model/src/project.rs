use serde::{Deserialize, Serialize};

use crate::{color::ColorInfo, edit::Edit, geometry::Size, ids::ProjectId, time::Time};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub format: FormatVersion,
    pub recording: RecordingInfo,
    pub meta: ProjectMeta,
    pub edit: Edit,
    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormatVersion {
    pub major: u16,
    pub minor: u16,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub title: String,
    pub tags: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordingInfo {
    pub size: Size,
    pub scale: f64,
    pub color: ColorInfo,
    pub audio_offset: Time,
    pub duration: Time,
    pub recorded_at: String,
    pub platform: String,
    pub video_codec: String,
    pub has_audio: AudioPresence,
    pub app_version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "bool", into = "bool")]
pub enum AudioPresence {
    Absent,
    Present,
}

impl From<bool> for AudioPresence {
    fn from(value: bool) -> Self {
        if value {
            Self::Present
        } else {
            Self::Absent
        }
    }
}

impl From<AudioPresence> for bool {
    fn from(value: AudioPresence) -> Self {
        value == AudioPresence::Present
    }
}
