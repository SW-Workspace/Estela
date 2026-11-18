# Estela

A screen recorder for Windows 10 2004+, Linux with PipeWire, and macOS.

Windows and Linux use Rust exclusively inside `wl/`. macOS will use only Swift
inside `mac/`, independently of the Rust workspace.

The Windows/Linux workspace includes an initial recording and preview milestone.
See [recording setup and limitations](wl/RECORDING.md). The project license has not been decided;
Cargo packages do not declare a distribution license.

```sh
cd wl
cargo build --workspace
cargo test --workspace
cargo xtask check-deps
cargo doc --workspace --no-deps
```

See [architecture](ARCHITECTURE.md), [development instructions](CONTRIBUTING.md),
and [recording setup](wl/RECORDING.md).
