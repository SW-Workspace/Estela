use windows::{
    core::GUID,
    Win32::Media::MediaFoundation::{
        eAVEncH264VProfile_Main, CODECAPI_AVEncMPVDefaultBPictureCount, CODECAPI_AVEncMPVGOPSize,
        IMFAttributes, IMFMediaType, MFCreateAttributes, MFCreateMediaType, MFMediaType_Video,
        MFNominalRange_16_235, MFVideoFormat_H264, MFVideoFormat_NV12,
        MFVideoInterlace_Progressive, MFVideoPrimaries_BT709, MFVideoTransFunc_709,
        MFVideoTransferMatrix_BT709, MF_MT_ALL_SAMPLES_INDEPENDENT, MF_MT_AVG_BITRATE,
        MF_MT_DEFAULT_STRIDE, MF_MT_FIXED_SIZE_SAMPLES, MF_MT_FRAME_RATE, MF_MT_FRAME_SIZE,
        MF_MT_INTERLACE_MODE, MF_MT_MAJOR_TYPE, MF_MT_MPEG2_PROFILE, MF_MT_PIXEL_ASPECT_RATIO,
        MF_MT_SAMPLE_SIZE, MF_MT_SUBTYPE, MF_MT_TRANSFER_FUNCTION, MF_MT_VIDEO_NOMINAL_RANGE,
        MF_MT_VIDEO_PRIMARIES, MF_MT_YUV_MATRIX, MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS,
        MF_SINK_WRITER_DISABLE_THROTTLING,
    },
};

use estela_media::recording::RecordingConfig;

use crate::{Error, Result};

const ATTRIBUTE_COMPONENT_BITS: u32 = 32;
const ATTRIBUTE_CAPACITY: u32 = 2;
const MIN_BITRATE: u64 = 2_000_000;
const MAX_BITRATE: u64 = 100_000_000;
const PIXEL_BITRATE_DIVISOR: u64 = 5;

pub(super) fn output_type(
    config: RecordingConfig,
    width: u32,
    height: u32,
) -> Result<IMFMediaType> {
    let media = video_type(config, width, height, MFVideoFormat_H264)?;
    let bitrate = (u64::from(width) * u64::from(height) * u64::from(config.fps)
        / PIXEL_BITRATE_DIVISOR)
        .clamp(MIN_BITRATE, MAX_BITRATE) as u32;
    // SAFETY: The H.264 type receives valid bitrate and profile values.
    unsafe {
        media.SetUINT32(&MF_MT_AVG_BITRATE, bitrate)?;
        media.SetUINT32(&MF_MT_MPEG2_PROFILE, eAVEncH264VProfile_Main.0 as u32)?;
    }
    Ok(media)
}

pub(super) fn input_type(config: RecordingConfig, width: u32, height: u32) -> Result<IMFMediaType> {
    let media = video_type(config, width, height, MFVideoFormat_NV12)?;
    let length = u64::from(width) * u64::from(height) * 3 / 2;
    let length = u32::try_from(length).map_err(|error| Error::Backend(error.to_string()))?;
    // SAFETY: NV12 samples use this exact stride and even-sized allocation.
    unsafe {
        media.SetUINT32(&MF_MT_DEFAULT_STRIDE, width)?;
        media.SetUINT32(&MF_MT_SAMPLE_SIZE, length)?;
        media.SetUINT32(&MF_MT_FIXED_SIZE_SAMPLES, 1)?;
        media.SetUINT32(&MF_MT_ALL_SAMPLES_INDEPENDENT, 1)?;
    }
    Ok(media)
}

fn video_type(
    config: RecordingConfig,
    width: u32,
    height: u32,
    subtype: GUID,
) -> Result<IMFMediaType> {
    // SAFETY: The caller owns a live MF runtime for the returned media type.
    let media = unsafe { MFCreateMediaType()? };
    // SAFETY: Values match the padded progressive BT.709 limited-range input.
    unsafe {
        media.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        media.SetGUID(&MF_MT_SUBTYPE, &subtype)?;
        media.SetUINT64(&MF_MT_FRAME_SIZE, packed(width, height))?;
        media.SetUINT64(&MF_MT_FRAME_RATE, packed(config.fps, 1))?;
        media.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, packed(1, 1))?;
        media.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)?;
        media.SetUINT32(&MF_MT_VIDEO_PRIMARIES, MFVideoPrimaries_BT709.0 as u32)?;
        media.SetUINT32(&MF_MT_TRANSFER_FUNCTION, MFVideoTransFunc_709.0 as u32)?;
        media.SetUINT32(&MF_MT_YUV_MATRIX, MFVideoTransferMatrix_BT709.0 as u32)?;
        media.SetUINT32(&MF_MT_VIDEO_NOMINAL_RANGE, MFNominalRange_16_235.0 as u32)?;
    }
    Ok(media)
}

pub(super) fn attributes() -> Result<IMFAttributes> {
    let mut attributes = None;
    // SAFETY: The output slot is writable and COM retains the returned object.
    unsafe { MFCreateAttributes(&mut attributes, ATTRIBUTE_CAPACITY)? };
    let attributes = attributes.ok_or_else(|| Error::Backend("Missing MF attributes".into()))?;
    // SAFETY: Both documented attributes accept UINT32 boolean values.
    unsafe {
        attributes.SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, 1)?;
        attributes.SetUINT32(&MF_SINK_WRITER_DISABLE_THROTTLING, 1)?;
    }
    Ok(attributes)
}

pub(super) fn encoding_parameters(config: RecordingConfig) -> Result<IMFAttributes> {
    let mut parameters = None;
    // SAFETY: The output slot is writable and remains alive for this call.
    unsafe { MFCreateAttributes(&mut parameters, ATTRIBUTE_CAPACITY)? };
    let parameters =
        parameters.ok_or_else(|| Error::Backend("Missing encoder parameters".into()))?;
    // SAFETY: H.264 parameters request one-second GOPs without B-frame delay.
    unsafe {
        parameters.SetUINT32(&CODECAPI_AVEncMPVGOPSize, config.fps)?;
        parameters.SetUINT32(&CODECAPI_AVEncMPVDefaultBPictureCount, 0)?;
    }
    Ok(parameters)
}

fn packed(high: u32, low: u32) -> u64 {
    (u64::from(high) << ATTRIBUTE_COMPONENT_BITS) | u64::from(low)
}
