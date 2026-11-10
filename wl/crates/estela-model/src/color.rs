use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorInfo {
    pub primaries: ColorPrimaries,
    pub range: ColorRange,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColorPrimaries {
    Bt709,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColorRange {
    Full,
    Limited,
}
