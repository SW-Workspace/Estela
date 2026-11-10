use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum GpuSelector {
    WindowsLuid(u64),
    LinuxRenderNode(PathBuf),
    Default,
}
