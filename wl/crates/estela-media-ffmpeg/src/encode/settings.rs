use std::{path::Path, process::Command};

use estela_media::recording::{RecordingBackend, RecordingConfig};

use crate::{Error, Result};

pub(super) fn command(
    path: &Path,
    config: RecordingConfig,
    backend: &RecordingBackend,
    input: &std::ffi::OsStr,
) -> Result<Command> {
    config
        .frame_bytes()
        .map_err(|error| Error::Backend(error.to_string()))?;
    let mut command = crate::runtime::command();
    command.args(["-hide_banner", "-loglevel", "error", "-nostdin", "-n"]);
    if let RecordingBackend::Vaapi { device } = backend {
        command.arg("-vaapi_device").arg(device);
    }
    command
        .args(["-f", "rawvideo", "-pixel_format", "rgba", "-video_size"])
        .arg(format!("{}x{}", config.width, config.height))
        .arg("-framerate")
        .arg(config.fps.to_string())
        .arg("-i")
        .arg(input)
        .arg("-an");
    encoding_settings(&mut command, backend)?;
    command
        .arg("-g")
        .arg(config.fps.to_string())
        .args([
            "-movflags",
            "+frag_keyframe+empty_moov+default_base_moof",
            "-f",
            "mp4",
        ])
        .arg(path);
    Ok(command)
}

fn encoding_settings(command: &mut Command, backend: &RecordingBackend) -> Result<()> {
    const PAD: &str = "pad=ceil(iw/2)*2:ceil(ih/2)*2";
    match backend {
        RecordingBackend::Software => {
            command.args([
                "-c:v", "mpeg4", "-q:v", "3", "-pix_fmt", "yuv420p", "-vf", PAD,
            ]);
        }
        RecordingBackend::Nvenc => {
            command.args([
                "-c:v",
                "h264_nvenc",
                "-preset",
                "p4",
                "-rc",
                "vbr",
                "-cq",
                "23",
                "-b:v",
                "0",
                "-pix_fmt",
                "yuv420p",
                "-vf",
                PAD,
            ]);
        }
        RecordingBackend::Vaapi { .. } => {
            command.args(["-c:v", "h264_vaapi", "-qp", "23", "-vf"]);
            command.arg(format!("{PAD},format=nv12,hwupload"));
        }
        RecordingBackend::MediaFoundation => {
            return Err(Error::Backend(
                "Media Foundation requires the Windows adapter".into(),
            ));
        }
    }
    Ok(())
}
