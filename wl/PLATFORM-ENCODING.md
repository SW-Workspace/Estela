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
| Windows | Software MPEG-4 | CPU RGBA | Fragmented MP4 |

Hardware candidates are capability hints rather than promises. FFmpeg encodes
three synthetic frames with the real dimensions and frame rate, then decodes one
frame and checks its size. Both subprocesses have bounded execution times and
write only into an owned temporary directory. No candidate can be chosen merely
because FFmpeg lists it. Preparation completes only after a candidate passes.

Initialization fallback does not imply recovery from an active encoder failure.
During recording, errors retain the partial file without publishing a completed
project. Existing destinations are rejected before opening an encoder.

## Native Windows integration

The next implementation supplies Media Foundation H.264 encoding from the
Windows platform crate. It must preserve fragmented media, explicit frame times,
clean finalization and cancellation. Hardware initialization must be verified
before selecting it, with software fallback if initialization fails.

The existing windows-capture encoder is not used as a shortcut: its public
container builder does not expose the fragmented MP4 sink needed for the current
partial recording guarantees. A direct Media Foundation adapter needs additional
features of the existing windows crate and encapsulated native API calls.

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
