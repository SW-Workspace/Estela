use crate::{Error, Result};

use super::RecordingConfig;

const RGBA_CHANNELS: usize = 4;
const BLOCK_PIXELS: i32 = 4;
const COLOR_SCALE_BITS: i32 = 16;
const COLOR_ROUNDING: i32 = 1 << (COLOR_SCALE_BITS - 1);
const LUMA_WEIGHTS: [i32; 3] = [11966, 40254, 4064];
const BLUE_CHROMA_WEIGHTS: [i32; 3] = [-6596, -22188, 28784];
const RED_CHROMA_WEIGHTS: [i32; 3] = [28784, -26145, -2639];
const LUMA_BLACK: u8 = 16;
const LUMA_WHITE: i32 = 235;
const CHROMA_NEUTRAL: u8 = 128;
const CHROMA_MAX: i32 = 240;

pub struct Nv12Buffer {
    source: RecordingConfig,
    width: u32,
    height: u32,
    bytes: Vec<u8>,
}

impl Nv12Buffer {
    pub fn new(source: RecordingConfig) -> Result<Self> {
        source.frame_bytes()?;
        let width = source.width.checked_add(1).map(|value| value & !1);
        let height = source.height.checked_add(1).map(|value| value & !1);
        let (Some(width), Some(height)) = (width, height) else {
            return Err(Error::Backend(
                "Recording dimensions cannot be padded".into(),
            ));
        };
        let pixels = (width as usize).checked_mul(height as usize);
        let length = pixels.and_then(|pixels| pixels.checked_add(pixels / 2));
        let length = length.ok_or_else(|| Error::Backend("NV12 frame is too large".into()))?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|error| Error::Backend(error.to_string()))?;
        bytes.resize(length, LUMA_BLACK);
        Ok(Self {
            source,
            width,
            height,
            bytes,
        })
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn convert(&mut self, rgba: &[u8]) -> Result<&[u8]> {
        if rgba.len() != self.source.frame_bytes()? {
            return Err(Error::Backend("Invalid RGBA frame length".into()));
        }
        let width = self.width as usize;
        let luma_length = width * self.height as usize;
        let (luma, chroma) = self.bytes.split_at_mut(luma_length);
        luma.fill(LUMA_BLACK);
        let source_stride = self.source.width as usize * RGBA_CHANNELS;
        convert_luma(rgba, luma, source_stride, width);
        convert_chroma(rgba, chroma, source_stride, width);
        Ok(&self.bytes)
    }
}

fn convert_luma(rgba: &[u8], luma: &mut [u8], source_stride: usize, destination_stride: usize) {
    let rows = rgba
        .chunks_exact(source_stride)
        .zip(luma.chunks_exact_mut(destination_stride));
    for (source, destination) in rows {
        let pixels = source.as_chunks::<RGBA_CHANNELS>().0;
        for (pixel, value) in pixels.iter().zip(destination.iter_mut()) {
            let rgb = [
                i32::from(pixel[0]),
                i32::from(pixel[1]),
                i32::from(pixel[2]),
            ];
            *value = component(rgb, LUMA_WEIGHTS, LUMA_BLACK, LUMA_WHITE);
        }
    }
}

fn convert_chroma(rgba: &[u8], chroma: &mut [u8], source_stride: usize, destination_stride: usize) {
    const PAIR_BYTES: usize = RGBA_CHANNELS * 2;
    let row_pair_bytes = source_stride.checked_mul(2).unwrap_or(rgba.len());
    let rows = rgba
        .chunks(row_pair_bytes)
        .zip(chroma.chunks_exact_mut(destination_stride));
    for (source, destination) in rows {
        let (top, bottom) = source.split_at(source_stride);
        let output = destination.as_chunks_mut::<2>().0;
        for (index, (pixels, uv)) in top.chunks(PAIR_BYTES).zip(output.iter_mut()).enumerate() {
            let start = index * PAIR_BYTES;
            let end = (start + PAIR_BYTES).min(bottom.len());
            let below = bottom.get(start..end).unwrap_or_default();
            let first = pair_sum(pixels);
            let second = pair_sum(below);
            let average = std::array::from_fn(|channel| {
                (first[channel] + second[channel] + BLOCK_PIXELS / 2) / BLOCK_PIXELS
            });
            uv[0] = component(average, BLUE_CHROMA_WEIGHTS, CHROMA_NEUTRAL, CHROMA_MAX);
            uv[1] = component(average, RED_CHROMA_WEIGHTS, CHROMA_NEUTRAL, CHROMA_MAX);
        }
    }
}

#[inline]
fn pair_sum(pixels: &[u8]) -> [i32; 3] {
    let Some(first) = pixels.get(..RGBA_CHANNELS) else {
        return [0; 3];
    };
    let mut sum = [
        i32::from(first[0]),
        i32::from(first[1]),
        i32::from(first[2]),
    ];
    if let Some(second) = pixels.get(RGBA_CHANNELS..RGBA_CHANNELS * 2) {
        sum[0] += i32::from(second[0]);
        sum[1] += i32::from(second[1]);
        sum[2] += i32::from(second[2]);
    }
    sum
}

#[inline]
fn component(pixel: [i32; 3], weights: [i32; 3], offset: u8, maximum: i32) -> u8 {
    let sum = pixel[0] * weights[0] + pixel[1] * weights[1] + pixel[2] * weights[2];
    let value = ((sum + COLOR_ROUNDING) >> COLOR_SCALE_BITS) + i32::from(offset);
    value.clamp(i32::from(LUMA_BLACK), maximum) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limited_range_preserves_black_white_and_rgba_channel_order() {
        let config = RecordingConfig {
            width: 2,
            height: 2,
            fps: 30,
        };
        let mut buffer = Nv12Buffer::new(config).unwrap();
        for (color, expected) in [
            ([0, 0, 0, 255], [16, 16, 16, 16, 128, 128]),
            ([255, 255, 255, 255], [235, 235, 235, 235, 128, 128]),
            ([255, 0, 0, 255], [63, 63, 63, 63, 102, 240]),
            ([0, 0, 255, 255], [32, 32, 32, 32, 240, 118]),
        ] {
            assert_eq!(buffer.convert(&color.repeat(4)).unwrap(), expected);
        }
    }

    #[test]
    fn odd_dimensions_pad_black_and_invalid_input_does_not_escape_validation() {
        let config = RecordingConfig {
            width: 1,
            height: 1,
            fps: 60,
        };
        let mut buffer = Nv12Buffer::new(config).unwrap();
        assert_eq!(buffer.size(), (2, 2));
        assert_eq!(
            buffer.convert(&[255; 4]).unwrap(),
            [235, 16, 16, 16, 128, 128]
        );
        assert!(buffer.convert(&[0; 3]).is_err());
        assert_eq!(buffer.convert(&[0; 4]).unwrap(), [16, 16, 16, 16, 128, 128]);
    }

    #[test]
    fn padded_rows_and_columns_preserve_chroma_block_alignment() {
        let config = RecordingConfig {
            width: 3,
            height: 3,
            fps: 30,
        };
        let mut buffer = Nv12Buffer::new(config).unwrap();
        let rgba = [255, 0, 0, 255].repeat(9);
        assert_eq!(
            buffer.convert(&rgba).unwrap(),
            [
                63, 63, 63, 16, 63, 63, 63, 16, 63, 63, 63, 16, 16, 16, 16, 16, 102, 240, 115, 184,
                115, 184, 122, 156,
            ]
        );
    }

    #[test]
    fn dimensions_and_timestamp_arithmetic_are_checked() {
        for config in [
            RecordingConfig {
                width: 0,
                height: 2,
                fps: 30,
            },
            RecordingConfig {
                width: 2,
                height: 2,
                fps: 0,
            },
            RecordingConfig {
                width: u32::MAX,
                height: u32::MAX,
                fps: 30,
            },
        ] {
            assert!(Nv12Buffer::new(config).is_err());
        }
        let config = RecordingConfig {
            width: 2,
            height: 2,
            fps: 60,
        };
        assert_eq!(config.timestamp(60).0, 1_000_000_000);
        assert_eq!(config.timestamp(u64::MAX).0, i64::MAX);
    }
}
