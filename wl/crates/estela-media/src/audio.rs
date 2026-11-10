use estela_model::Time;

#[derive(Debug, Clone, Copy)]
pub struct AudioFormat {
    pub sample_rate: u32,
    pub channels: u16,
}
pub struct AudioChunk {
    pub pts: Time,
    pub format: AudioFormat,
    pub samples: Vec<f32>,
}
