# Architecture

The complete Windows and Linux Rust workspace lives in `wl/`, including the
shared core, UI, CLI, platform adapters, configuration, tests and tooling.
Core crates are shared only between Windows and Linux.

macOS will use an independent Swift implementation in `mac/`, without Rust
dependencies. macOS work is outside the scope of Windows and Linux tasks.

The UI and CLI call the engine and library. The engine does not depend on the
library. Adapters depend only on contracts. Allowed edges are defined in
`wl/xtask/dependencies.json` and checked from `wl/` with `cargo xtask check-deps`.

See [layer details](wl/reference/architecture.md) and
[architecture decisions](wl/reference/adr/).
