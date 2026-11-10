use std::path::{Path, PathBuf};

pub fn executable(bin_dir: &Path, name: &str) -> PathBuf {
    bin_dir.join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
}
