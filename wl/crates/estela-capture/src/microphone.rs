use std::time::Instant;

pub const SAMPLE_RATE: u32 = 48_000;
pub const SAMPLE_BYTES: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Input {
    #[default]
    Muted,
    Default,
    Device(String),
}

pub struct Packet {
    pub start: Instant,
    pub pcm: Vec<u8>,
}
