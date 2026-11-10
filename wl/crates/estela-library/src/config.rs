use std::path::PathBuf;

pub enum LibraryConfig {
    Bundled { bin_dir: PathBuf, data_dir: PathBuf },
    // URLs may contain passwords, so they must never be logged.
    External { url: String },
}
