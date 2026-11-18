# Windows and Linux

This directory contains the entire Rust workspace. Run `cargo build --workspace`
from here. Platform crates compile empty outside their target OS. macOS will use
Swift independently in `mac/`.

Pending native integrations require dynamically linked LGPL FFmpeg, PostgreSQL
16+ with ICU, and the appropriate system SDKs. Linux needs PipeWire, ALSA, Vulkan
and xdg portals development support. Windows needs the Windows SDK and MSVC.

Packaging directories contain templates, not distributable installers.
See [recording setup and implementation status](RECORDING.md).
