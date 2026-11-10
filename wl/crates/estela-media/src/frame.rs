use estela_model::{ColorInfo, Time};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Nv12,
    P010,
    Bgra8,
}
#[derive(Debug, Clone)]
pub struct CpuPlanes {
    pub planes: Vec<Vec<u8>>,
    pub strides: Vec<u32>,
}
#[derive(Debug, Clone, Copy)]
/// The owner must keep the native resource alive while this pointer is used.
pub struct RawPtr(pub *mut std::ffi::c_void);
#[derive(Debug)]
pub struct DmaBufDescriptor {
    pub fds: Vec<i32>,
    pub strides: Vec<u32>,
    pub offsets: Vec<u32>,
    pub modifier: u64,
}
#[derive(Debug)]
pub enum NativeFrame {
    D3D11 { texture: RawPtr, array_index: u32 },
    DrmPrime(DmaBufDescriptor),
    Cpu(CpuPlanes),
}
#[derive(Debug)]
pub struct GpuFrame {
    pub pts: Time,
    pub size: (u32, u32),
    pub format: PixelFormat,
    pub color: ColorInfo,
    pub planes: FramePlanes,
}
#[derive(Debug)]
pub enum FramePlanes {
    Cpu(CpuPlanes),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKind {
    Keyframe,
    Interframe,
}
