use std::path::Path;

use crate::{runtime, Error, Result};

pub fn microphone(video: &Path, pcm: &Path, output: &Path, sample_rate: u32) -> Result<()> {
    let mut command = runtime::command();
    command
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-n", "-i"])
        .arg(video)
        .args(["-f", "s16le", "-ar"])
        .arg(sample_rate.to_string())
        .args(["-ac", "1", "-i"])
        .arg(pcm)
        .args([
            "-map",
            "0:v:0",
            "-map",
            "1:a:0",
            "-c:v",
            "copy",
            "-c:a",
            "aac",
            "-movflags",
            "+frag_keyframe+delay_moov+default_base_moof",
            "-avoid_negative_ts",
            "disabled",
            "-f",
            "mp4",
        ])
        .arg(output);
    let result = command
        .output()
        .map_err(|error| runtime::launch_error(&command, error))?;
    if !result.status.success() {
        return Err(Error::Backend(format!(
            "FFmpeg could not add the microphone track: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        )));
    }
    Ok(())
}
