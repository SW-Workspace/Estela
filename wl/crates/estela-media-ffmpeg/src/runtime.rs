use std::{
    ffi::OsString,
    path::Path,
    process::{Child, Command},
};

use crate::{Error, Result};

pub fn executable() -> OsString {
    resolve(
        std::env::var_os("ESTELA_FFMPEG"),
        std::env::current_exe().ok().as_deref(),
    )
}

fn resolve(override_path: Option<OsString>, application: Option<&Path>) -> OsString {
    if let Some(path) = override_path {
        return path;
    }
    let name = format!("ffmpeg{}", std::env::consts::EXE_SUFFIX);
    if let Some(directory) = application.and_then(Path::parent) {
        for candidate in [directory.join(&name), directory.join("ffmpeg").join(&name)] {
            if candidate.is_file() {
                return candidate.into_os_string();
            }
        }
    }
    name.into()
}

pub fn spawn(command: &mut Command) -> Result<Child> {
    command
        .spawn()
        .map_err(|error| launch_error(command, error))
}

pub fn launch_error(command: &Command, error: std::io::Error) -> Error {
    Error::Backend(format!(
        concat!(
            "Could not run FFmpeg at {}: {}. ",
            "Place FFmpeg beside Estela, install it on PATH, ",
            "or set ESTELA_FFMPEG to its executable path",
        ),
        command.get_program().to_string_lossy(),
        error,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_override_has_priority() {
        assert_eq!(
            resolve(Some("custom-ffmpeg".into()), None),
            OsString::from("custom-ffmpeg"),
        );
    }

    #[test]
    fn falls_back_to_path_without_a_bundle() {
        assert_eq!(
            resolve(None, None),
            OsString::from(format!("ffmpeg{}", std::env::consts::EXE_SUFFIX)),
        );
    }

    #[test]
    fn bundle_lookup_does_not_depend_on_working_directory() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "estela-ffmpeg-runtime-{}-{unique}",
            std::process::id(),
        ));
        let bundle = directory.join("ffmpeg");
        std::fs::create_dir_all(&bundle).unwrap();
        let name = format!("ffmpeg{}", std::env::consts::EXE_SUFFIX);
        let nested = bundle.join(&name);
        std::fs::write(&nested, []).unwrap();
        let application = directory.join("estela-app");
        assert_eq!(resolve(None, Some(&application)), nested.into_os_string());
        std::fs::remove_dir_all(bundle).unwrap();
        let adjacent = directory.join(name);
        std::fs::write(&adjacent, []).unwrap();
        assert_eq!(resolve(None, Some(&application)), adjacent.into_os_string());
        assert_eq!(
            resolve(Some("override".into()), Some(&application)),
            OsString::from("override"),
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn launch_failure_identifies_ffmpeg_and_configuration() {
        let command = Command::new("missing-ffmpeg");
        let error = launch_error(&command, std::io::ErrorKind::NotFound.into()).to_string();
        assert!(error.contains("missing-ffmpeg"));
        assert!(error.contains("ESTELA_FFMPEG"));
    }
}
