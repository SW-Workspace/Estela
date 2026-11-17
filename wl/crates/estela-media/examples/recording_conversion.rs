use std::{hint::black_box, time::Instant};

use estela_media::recording::{nv12::Nv12Buffer, RecordingConfig};

const BENCHMARK_FRAMES: u32 = 120;
const FRAME_RATE: u32 = 60;
const FRAME_COLOR: [u8; 4] = [220, 60, 20, 255];
const DIMENSIONS: [(u32, u32); 2] = [(1280, 720), (1920, 1080)];
const MILLISECONDS_PER_SECOND: f64 = 1000.0;

fn main() -> estela_media::Result<()> {
    for (width, height) in DIMENSIONS {
        let config = RecordingConfig {
            width,
            height,
            fps: FRAME_RATE,
        };
        let rgba = FRAME_COLOR.repeat(config.frame_bytes()? / FRAME_COLOR.len());
        let mut buffer = Nv12Buffer::new(config)?;
        let started = Instant::now();
        for _ in 0..BENCHMARK_FRAMES {
            black_box(buffer.convert(black_box(&rgba))?);
        }
        let seconds = started.elapsed().as_secs_f64();
        let frames = f64::from(BENCHMARK_FRAMES);
        println!(
            "{width}x{height}: {:.2} ms/frame, {:.1} frames/s (conversion only)",
            seconds * MILLISECONDS_PER_SECOND / frames,
            frames / seconds,
        );
    }
    Ok(())
}
