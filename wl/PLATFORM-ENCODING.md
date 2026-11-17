# Platform recording encoders

The Windows and Linux recording engine shares orchestration, project metadata,
pause/resume, cursor events and the nanosecond recording clock. Capture adapters
remain isolated by operating system. The recording encoder is a separate backend
selected before recording starts.

## Contracts and ownership

`estela-media::recording` owns the configuration, backend identifiers, timestamped
RGBA input and the object-safe recording encoder contract. The existing packet
encoder contract remains available for future GPU rendering and export pipelines.
No platform handles or platform dependencies enter the UI or shared media crate.

Platform adapters supply ordered backend candidates. The engine connects those
candidates to the implementation and owns the encoder on the recording worker.
FFmpeg implements the recording contract without depending on either platform.
The dependency graph remains unchanged.

The engine sends consecutive `Time` values computed from the accepted frame
count. Pauses do not advance that count. FFmpeg checks those values against its
constant frame-rate input, rejecting mismatched timestamps and malformed frames.
The native encoder must obey the same clock, including repeated frames during
slow capture.

## Current implementations

| Platform | Candidates | Input | Output |
| --- | --- | --- | --- |
| Linux | NVENC when an NVIDIA node is present, VA-API for sorted DRM render nodes, software MPEG-4 | CPU RGBA | Fragmented MP4 |
| Windows | Media Foundation H.264, software MPEG-4 | CPU RGBA converted to NV12 | Fragmented MP4 |

Hardware candidates are capability hints rather than promises. FFmpeg encodes
three synthetic frames with the real dimensions and frame rate, then decodes one
frame and checks its size. Both FFmpeg subprocesses have bounded execution times and
write only into an owned temporary directory. No candidate can be chosen merely
because FFmpeg lists it. Preparation completes only after a candidate passes.

Initialization fallback does not imply recovery from an active encoder failure.
During recording, errors retain the partial file without publishing a completed
project. Existing destinations are rejected before opening an encoder.

## Native Windows integration

Media Foundation H.264 encoding lives in the Windows platform crate, behind the
same recording contract. Only the existing windows crate gains API features;
there are no new packages or dependency graph edges. Each native API boundary
has an explicit safety justification.

The adapter initializes COM and Media Foundation on the owning thread. A runtime
guard cannot move between threads. Shutdown releases the sink writer before the
media sink, closes the byte stream, and finally balances MF and COM startup.
Initialization failures remove only a destination created by that attempt.
Existing files use the fail-if-exists native mode and are never removed.

A fragmented MP4 sink is created explicitly with `MFCreateFMPEG4MediaSink`, so
output does not depend on the `.partial.estela` extension. NV12 conversion uses
limited-range BT.709, pads odd source dimensions to even coded dimensions, and
checks sample size. Frame times and durations derive from the shared nanosecond
clock and are converted to 100-nanosecond MF ticks without cumulative drift.
H.264 requests a one-second GOP and no B-frame delay.

`MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS` permits hardware MFT selection. It does
not prove that the selected transform is hardware: Media Foundation can choose
its native software encoder. A failed native capability probe selects the shared
FFmpeg software fallback. A failure to open the real native destination retries
remaining candidates only if no destination remains from the failed attempt.

The native capability probe creates and finalizes three actual frames, then
checks the resulting MP4 boxes for movie metadata, fragments and media data.
It runs on a separate COM thread with a ten-second response deadline. A timed-out
thread is detached because Rust cannot terminate a driver call safely; it owns
its resources and cleans them up if the call eventually returns. This deadline
does not bound native driver calls during an active recording.

Reference API contracts:

- [Fragmented MP4 sink](https://learn.microsoft.com/en-us/windows/win32/api/mfidl/nf-mfidl-mfcreatefmpeg4mediasink).
- [Sink writer ownership](https://learn.microsoft.com/en-us/windows/win32/api/mfreadwrite/nf-mfreadwrite-mfcreatesinkwriterfrommediasink).
- [Hardware transform selection](https://learn.microsoft.com/en-us/windows/win32/medfound/mf-readwrite-enable-hardware-transforms).

## GPU transport

Capture still reads frames into CPU memory. Direct3D textures and DMA-BUF handles
must eventually carry explicit ownership, synchronization and resource lifetime
across capture, conversion and encoding. The initial encoder separation does not
claim zero-copy capture or hardware throughput on untested machines.

## Validation

Run the normal workspace checks and cross-check the Windows target. The explicit
engine tests also exercise unavailable hardware, software fallback, malformed
frames, timestamp validation, odd dimensions, destination preservation, slow
capture, pause/resume and finalization:

```sh
cargo test -p estela-engine --lib -- --include-ignored
cargo test -p estela-media-ffmpeg --test runtime -- --ignored
```

A hardware throughput claim additionally requires recording and reopening output
on actual Intel, AMD and NVIDIA devices. Native Windows API behavior must be
verified on Windows; cross-compilation alone cannot establish it.

On Windows, run the explicit native round trip with a shared LGPL FFmpeg decoder:

```powershell
./ci/check-windows-encoding.ps1 -FfmpegDirectory C:/Dependencies/ffmpeg/bin
```

This verifies H.264 fragmented output, 30/60 fps, exact decoded frame counts,
RGBA channel order, odd dimensions, invalid samples and destination protection.
The test is explicit because it requires native Media Foundation and the decoder;
it cannot be executed by a Linux cross-compilation check.

## CPU conversion benchmark

The NV12 converter processes luma in contiguous rows and chroma in paired rows,
so the compiler can optimize the dense pixel path without unsafe code. The odd
row and column cases retain explicit black padding and correct chroma averaging.
A regression test checks the resulting planes at an odd-sized colored boundary.

Run the same synthetic conversion workload on either operating system:

```sh
cargo run --release -p estela-media --example recording_conversion
```

The example reports milliseconds per frame and conversion throughput at 720p and
1080p. It excludes capture, synchronization, encoding and file I/O; its throughput
is not a claim about end-to-end recording FPS or native hardware acceleration.
