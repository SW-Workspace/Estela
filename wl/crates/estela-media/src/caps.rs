#[derive(Debug, Clone)]
pub struct CodecProfile {
    pub codec: String,
    pub profile: String,
    pub accelerator: String,
}
#[derive(Debug, Clone, Default)]
pub struct Capabilities {
    pub accelerators: Vec<String>,
    pub decode: Vec<CodecProfile>,
    pub encode: Vec<CodecProfile>,
}
