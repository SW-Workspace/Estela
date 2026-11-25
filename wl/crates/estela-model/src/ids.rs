use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub ulid::Ulid);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElementId(pub ulid::Ulid);

impl ElementId {
    pub fn new() -> Self {
        Self(ulid::Ulid::new())
    }
}

impl Default for ElementId {
    fn default() -> Self {
        Self::new()
    }
}
