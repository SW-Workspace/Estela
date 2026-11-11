# Recording and preview milestone

This milestone records a screen or window, saves one local `.estela` file and
plays it inside Estela with automatic zoom. Manual editing, export, microphone
capture, hardware encoding and the database library are outside this milestone.
The original media remains unchanged when automatic zoom is toggled in preview.

## Run

Use the Rust version in `rust-toolchain.toml`. From `wl/`:

```sh
cargo run -p estela-app
cargo run -p estela-app -- /path/to/recording.estela
```

Install a dynamically linked **LGPL** FFmpeg build on `PATH`, or set
`ESTELA_FFMPEG` to its executable path. The initial recording encoder uses
FFmpeg's native MPEG-4 encoder rather than a GPL encoder. FFmpeg is a runtime
requirement and is not downloaded or bundled by the application.

Linux also requires PipeWire development headers, libclang, a C compiler,
pkg-config, a working graphics driver and xdg-desktop-portal with the desktop's
ScreenCast and GlobalShortcuts backends. For example:

```sh
# Fedora.
sudo dnf install pipewire-devel clang clang-devel pkgconf-pkg-config gcc ffmpeg-free

# Ubuntu.
sudo apt install libpipewire-0.3-dev libclang-dev pkg-config build-essential \
    libxkbcommon-dev libwayland-dev mesa-vulkan-drivers ffmpeg
```

Windows requires the Windows SDK and the MSVC Rust toolchain for the normal
native build. Capture uses Windows Graphics Capture on Windows 10 2004 or newer.
The workspace can also be checked for the Windows GNU target from Linux.

## Flow

1. Choose Screen or Window and a source. Wayland offers a direct Choose window/display action that opens the system
   picker before recording. The prepared session is reused when Record starts;
   selecting a source does not create a recording file. The portal controls which
   sources are available.
2. Choose 30 or 60 fps and whether to use the three-second countdown.
3. Record, pause/resume, restart or stop. Escape cancels source preparation and
   the countdown. Restart discards the current take and starts selection again.
4. Stopping saves the recording in the user's Videos/Estela directory and opens
   preview. Open recording also reopens an existing `.estela` file.
5. Preview supports play/pause, seeking, frame stepping, full screen and an
   automatic zoom switch. There are no editing or export controls.

The interface supports English and Spanish using the existing locale precedence.
The recorder remembers its position, countdown preference and frame rate.
Inter is bundled with its OFL license.

## Desktop differences

| Capability | Windows | Linux X11 | Linux Wayland |
| --- | --- | --- | --- |
| Source selection | Monitor/window list | Monitor/window list | System ScreenCast picker |
| Capture | Windows Graphics Capture | X11 and Composite window pixmaps | Portal and PipeWire |
| Automatic zoom | Mouse clicks | Mouse button transitions | Cursor dwell when metadata is available |
| Cursor | Recorded separately | Recorded separately | Metadata when supported; embedded fallback |
| Exclude recorder from full-screen capture | Window display affinity | Hide recorder | Hide recorder |
| Global controls | Native hotkeys | X11 key grabs | GlobalShortcuts portal |

Default shortcuts are **Ctrl+Shift+8** to start/stop and **Ctrl+Shift+9** to
pause/resume. The Wayland compositor may ask for shortcut permission or choose a
different binding. Full-screen capture without window exclusion requires working
global controls before recording is enabled. Pausing restores the controls only
after the recording worker acknowledges the pause.

Wayland does not expose global clicks through ScreenCast. Cursor dwell is an
explicitly identified alternative, not a fabricated click log. If the compositor
only supplies an embedded cursor, automatic zoom is unavailable and the interface
explains that limitation. X11 window capture uses a Composite pixmap so overlapping
controls are not included in the selected window's image.

## Persistence and failures

A take is written as fragmented media to a unique `.partial.estela` file. A clean
stop finalizes metadata with checksums and two metadata indexes, syncs the file,
and renames it to `.estela`. Existing destinations are never intentionally
replaced. The reader checks metadata bounds and checksums and falls back to the
other index if one is damaged.

Capture/encoder failure retains the partial media and reports an error. Partial
files are not listed as completed recordings and cannot yet be repaired through
the interface. Closing during an active recording requests a clean stop first.
Changing source dimensions ends capture with an error rather than writing frames
with an incompatible layout. Playback errors are surfaced in the interface.

This is an initial implementation, not a published interoperability specification
or a completed implementation of the broader editor and library architecture.

## Validation

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
cargo xtask check-deps
cargo xtask check-sqlx
```

With FFmpeg installed, run the explicit media integration checks:

```sh
cargo test -p estela-engine --lib -- --ignored
cargo test -p estela-engine --test recording_roundtrip -- --ignored
```

The X11 overlap regression must run on an isolated server, never the real desktop:

```sh
Xvfb :97 -screen 0 1280x900x24 -nolisten tcp &
env -u WAYLAND_DISPLAY DISPLAY=:97 ESTELA_TEST_X11=1 \
    cargo test -p estela-platform-linux --test x11_capture -- --ignored
```

On both real desktops, verify display and window selection, canceled permission
requests, countdown cancellation, pause/resume, restart, stopping via shortcuts,
reopening a saved recording, seeking, automatic zoom, and source closure. On
Windows, confirm the floating controls are absent from a full-screen recording.
On Wayland, check the actual compositor's cursor metadata and shortcut support.

The dependency audit has scoped maintenance exceptions for RUSTSEC-2026-0192
(`ttf-parser`, through cosmic-text) and RUSTSEC-2026-0206 (`rustybuzz`, through
Iced's SVG renderer). Both are inherited from pinned Iced and have no safe upgrade
in the current dependency chain. Other advisories remain checked.

The platform adapter selects WGPU on Windows and Wayland for alpha-capable
presentation, and tiny-skia on X11 together with native bounding/input shapes.
Mesa's software Vulkan driver can be used on Wayland without a hardware GPU.
Tiny-skia/softbuffer is not used on Wayland because its XRGB buffers lose alpha.

## Recorder controls on Linux

The source selector, microphone status, Record button and options menu stay in
one persistent row. Options open below it: screen/window selection, microphone
status, frame rate/countdown settings, shortcuts and opening existing recordings.
The microphone remains explicitly muted in this video-only milestone.

The recorder keeps one transparent surface throughout its lifetime. Panels are
centered below the bar without resizing or replacing its native window. Preview
uses a separate window while retaining the recorder and its clipboard connection.
Native Wayland input regions, X11 shapes and Windows regions pass pointer input
to underlying applications outside the visible controls. Dragging starts on
pointer press at the grip or unused bar background and follows the compositor's
window placement rules. Icons come directly from the root assets/icons directory.
Missing shortcut permissions are explained below the controls.

The Linux portal connection uses a persistent runtime, including after failed
shortcut registration. Host applications register their application identity when
the portal supports it.

An isolated visual regression check exercises the recorder's option panels,
verifies that Record remains visible, and exits through the options menu. The
Wayland check also drags the bar, checks focus through transparent regions,
stresses clipboard offers, and rejects native protocol errors:

```sh
cargo build -p estela-app
python3 ci/check-recorder-ui.py
python3 ci/check-recorder-ui.py --wayland
```

The check requires Xvfb, FFmpeg and xdotool; the Wayland variant also requires
Weston and wl-clipboard. It creates its own display, runtime and configuration directories, and
uses an unavailable test D-Bus address to exercise the permission failure state.
Screenshots and logs are written under `target/recorder-x11/` and
`target/recorder-wayland/`. It does not interact with the user's desktop.
