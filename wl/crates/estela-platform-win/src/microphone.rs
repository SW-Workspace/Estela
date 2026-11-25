use std::{
    ptr,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::SyncSender,
    },
    time::{Duration, Instant},
};

use windows::{
    core::{PCWSTR, PWSTR},
    Win32::{
        Devices::FunctionDiscovery::PKEY_Device_FriendlyName,
        Foundation::{CloseHandle, HANDLE, WAIT_FAILED},
        Media::Audio::{
            eCapture, eConsole, IAudioCaptureClient, IAudioClient, IMMDevice, IMMDeviceEnumerator,
            MMDeviceEnumerator, AUDCLNT_BUFFERFLAGS_SILENT, AUDCLNT_BUFFERFLAGS_TIMESTAMP_ERROR,
            AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM,
            AUDCLNT_STREAMFLAGS_EVENTCALLBACK, AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY,
            DEVICE_STATE_ACTIVE, WAVEFORMATEX, WAVE_FORMAT_PCM,
        },
        System::{
            Com::{
                CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize,
                StructuredStorage::{PropVariantClear, PropVariantToStringAlloc},
                CLSCTX_ALL, COINIT_MULTITHREADED, STGM_READ,
            },
            Performance::{QueryPerformanceCounter, QueryPerformanceFrequency},
            Threading::{CreateEventW, WaitForSingleObject},
        },
    },
};

use estela_capture::microphone::{Device, Packet, SAMPLE_BYTES, SAMPLE_RATE};

use crate::{Error, Result};

const STOP_CHECK_MS: u32 = 20;
const TICKS_PER_SECOND: u128 = 10_000_000;

struct Apartment;

impl Apartment {
    fn enter() -> Result<Self> {
        // SAFETY: Each successful initialization is balanced on this thread.
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).ok()? };
        Ok(Self)
    }
}

impl Drop for Apartment {
    fn drop(&mut self) {
        // SAFETY: This guard exists only after successful initialization.
        unsafe { CoUninitialize() };
    }
}

pub fn devices() -> Result<Vec<Device>> {
    let _apartment = Apartment::enter()?;
    let enumerator = enumerator()?;
    // SAFETY: COM is initialized and the collection is used on this thread.
    let collection = unsafe { enumerator.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)? };
    // SAFETY: The collection owns a valid COM interface.
    let count = unsafe { collection.GetCount()? };
    (0..count)
        .map(|index| {
            // SAFETY: Index is within the collection returned by Windows.
            let device = unsafe { collection.Item(index)? };
            // SAFETY: GetId returns a COM-allocated, terminated string on success.
            let id = owned_string(unsafe { device.GetId()? })?;
            let name = device_name(&device).unwrap_or_else(|_| id.clone());
            Ok(Device { id, name })
        })
        .collect()
}

fn device_name(device: &IMMDevice) -> Result<String> {
    // SAFETY: The endpoint owns a valid COM interface.
    let store = unsafe { device.OpenPropertyStore(STGM_READ)? };
    // SAFETY: The key is a valid Windows device property key.
    let mut value = unsafe { store.GetValue(&PKEY_Device_FriendlyName)? };
    // SAFETY: The property remains valid until PropVariantClear.
    let name = unsafe { PropVariantToStringAlloc(&value) };
    // SAFETY: Clear releases the property allocated by GetValue.
    let cleared = unsafe { PropVariantClear(&mut value) };
    let name = owned_string(name?)?;
    cleared?;
    Ok(name)
}

fn owned_string(value: PWSTR) -> Result<String> {
    // SAFETY: The caller passes a COM-allocated, terminated string.
    let string = unsafe { value.to_string() };
    // SAFETY: The allocation belongs to the COM task allocator.
    unsafe { CoTaskMemFree(Some(value.0.cast())) };
    string.map_err(|error| Error::Backend(error.to_string()))
}

fn enumerator() -> Result<IMMDeviceEnumerator> {
    // SAFETY: The calling thread holds a COM apartment guard.
    Ok(unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? })
}

pub fn capture(target: Option<&str>, packets: SyncSender<Packet>, stop: &AtomicBool) -> Result<()> {
    let _apartment = Apartment::enter()?;
    let enumerator = enumerator()?;
    let device = if let Some(target) = target {
        let target: Vec<u16> = target.encode_utf16().chain([0]).collect();
        // SAFETY: The endpoint ID is terminated and lives through the call.
        unsafe { enumerator.GetDevice(PCWSTR(target.as_ptr()))? }
    } else {
        // SAFETY: The enumerator belongs to the calling COM apartment.
        unsafe { enumerator.GetDefaultAudioEndpoint(eCapture, eConsole)? }
    };
    let ready = ReadyEvent::new()?;
    let client = open(&device, &ready)?;
    // SAFETY: The initialized client supports the capture service.
    let capture: IAudioCaptureClient = unsafe { client.GetService()? };
    let clock = Clock::new()?;
    // SAFETY: The capture stream is initialized and runs on this thread.
    unsafe { client.Start()? };
    while !stop.load(Ordering::Relaxed) {
        ready.wait()?;
        // SAFETY: This thread owns the capture interface and all its buffers.
        while unsafe { capture.GetNextPacketSize()? } != 0 {
            let _ = packets.try_send(read_packet(&capture, &clock)?);
        }
    }
    // SAFETY: This thread started and owns the stream.
    unsafe { client.Stop()? };
    Ok(())
}

fn open(device: &IMMDevice, ready: &ReadyEvent) -> Result<IAudioClient> {
    // SAFETY: WASAPI activation runs in the endpoint's COM apartment.
    let client: IAudioClient = unsafe { device.Activate(CLSCTX_ALL, None)? };
    let format = WAVEFORMATEX {
        wFormatTag: WAVE_FORMAT_PCM as u16,
        nChannels: 1,
        nSamplesPerSec: SAMPLE_RATE,
        nAvgBytesPerSec: SAMPLE_RATE * SAMPLE_BYTES as u32,
        nBlockAlign: SAMPLE_BYTES as u16,
        wBitsPerSample: (SAMPLE_BYTES * 8) as u16,
        cbSize: 0,
    };
    // SAFETY: The PCM descriptor is complete and lives through initialization.
    unsafe {
        client.Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM
                | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY
                | AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
            0,
            0,
            &format,
            None,
        )?;
    }
    // SAFETY: The event outlives the audio client.
    unsafe { client.SetEventHandle(ready.0)? };
    Ok(client)
}

fn read_packet(capture: &IAudioCaptureClient, clock: &Clock) -> Result<Packet> {
    let mut data = ptr::null_mut();
    let (mut frames, mut flags, mut qpc) = (0, 0, 0);
    // SAFETY: The output pointers are valid and the packet is ready.
    unsafe { capture.GetBuffer(&mut data, &mut frames, &mut flags, None, Some(&mut qpc))? };
    let bytes = frames as usize * SAMPLE_BYTES;
    let pcm = if data.is_null() || flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0 {
        vec![0; bytes]
    } else {
        // SAFETY: WASAPI owns these PCM bytes until ReleaseBuffer.
        unsafe { std::slice::from_raw_parts(data, bytes) }.to_vec()
    };
    // SAFETY: This releases exactly the frames acquired by GetBuffer.
    unsafe { capture.ReleaseBuffer(frames)? };
    let start = if flags & AUDCLNT_BUFFERFLAGS_TIMESTAMP_ERROR.0 as u32 == 0 {
        clock.instant(qpc)
    } else {
        Instant::now() - Duration::from_secs_f64(f64::from(frames) / f64::from(SAMPLE_RATE))
    };
    Ok(Packet { start, pcm })
}

struct ReadyEvent(HANDLE);

impl ReadyEvent {
    fn new() -> Result<Self> {
        // SAFETY: An unnamed event uses default security and has a single owner.
        Ok(Self(unsafe {
            CreateEventW(None, false, false, PCWSTR::null())?
        }))
    }

    fn wait(&self) -> Result<()> {
        // SAFETY: The event remains open for the duration of this wait.
        if unsafe { WaitForSingleObject(self.0, STOP_CHECK_MS) } == WAIT_FAILED {
            return Err(windows::core::Error::from_win32().into());
        }
        Ok(())
    }
}

impl Drop for ReadyEvent {
    fn drop(&mut self) {
        // SAFETY: This guard owns the event and closes it exactly once.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

struct Clock {
    instant: Instant,
    ticks: u64,
}

impl Clock {
    fn new() -> Result<Self> {
        let (mut frequency, mut counter) = (0, 0);
        // SAFETY: Windows initializes the supplied frequency and counter.
        unsafe {
            QueryPerformanceFrequency(&mut frequency)?;
            QueryPerformanceCounter(&mut counter)?;
        }
        Ok(Self {
            instant: Instant::now(),
            ticks: (counter as u128 * TICKS_PER_SECOND / frequency as u128) as u64,
        })
    }

    fn instant(&self, ticks: u64) -> Instant {
        let delta = Duration::from_nanos(ticks.abs_diff(self.ticks).saturating_mul(100));
        if ticks >= self.ticks {
            self.instant + delta
        } else {
            self.instant - delta
        }
    }
}
