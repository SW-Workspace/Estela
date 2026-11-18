# AGENTS.md

Rules for any agent or contributor working on Estela for **Windows and Linux**.
macOS uses **Swift only**, never Rust, and is out of scope for this file.
macOS work follows [mac/AGENTS.md](mac/AGENTS.md).
All Rust code, crates, build configuration and tooling live inside `wl/`.
This file lives at the repository root. There is no `docs/` directory for now;
keep architecture and development documentation in root or `wl/` Markdown files.

## 1. Scope

| Path | Status for agents |
| --- | --- |
| `wl/crates/` | Windows and Linux crates, including their shared core. Editable only for tasks that need it (see section 3). |
| `wl/` | All Rust code, build configuration, tooling, tests, packaging and CI scripts. Main work area. |
| `mac/` | **Off limits.** Swift only. Do not create, edit, move or delete anything here. |
| `assets/`, `.github/`, root Markdown files | Editable when the task requires it. |

Never add macOS code paths anywhere: no `cfg(target_os = "macos")` branches,
no Mac crates, no Mac CI jobs. Never create a root Rust workspace or put Rust
crates, configuration, tests or tooling outside `wl/`.

## 2. Language

Everything in the repository is written in **English**:

- Identifiers, modules, crates, files and folders.
- Comments and doc comments.
- Commit messages, branch names, pull request titles and descriptions.
- Log messages, error messages and documentation.

User-facing interface text is the only exception: it lives in localization files
(English as the source language, Spanish as a translation), never hard-coded in Rust.

## 3. Operating system isolation

When a task targets one operating system, change only that system's files.

| Task targets | May edit | Must not edit |
| --- | --- | --- |
| Windows | `wl/crates/estela-platform-win/`, `wl/packaging/windows/`, Windows scripts in `wl/ci/` | `wl/crates/estela-platform-linux/`, `wl/packaging/linux/`, Linux scripts in `wl/ci/`, `mac/` |
| Linux | `wl/crates/estela-platform-linux/`, `wl/packaging/linux/`, Linux scripts in `wl/ci/` | `wl/crates/estela-platform-win/`, `wl/packaging/windows/`, Windows scripts in `wl/ci/`, `mac/` |
| Both systems | Shared crates in `wl/crates/`, including `estela-app`; both platform crates when the task names both | `mac/` |

Rules:

1. If an OS-specific task seems to need a change in a shared crate or in the other OS, **stop and report it**. That change goes in its own branch and pull request.
2. A change in a shared crate must keep the whole workspace building and passing tests on **both** Windows and Linux.
3. Do not reformat, rename or "clean up" files outside the task, even if they are in the same crate.
4. `cfg(windows)` and `cfg(target_os = "linux")` are allowed only in `wl/crates/estela-engine/src/platform.rs` and inside the platform crates.

## 4. Code style

The code follows the [Rust Style Guide](https://doc.rust-lang.org/style-guide/),
enforced with `rustfmt` default settings. Key rules:

- Spaces, never tabs; 4 spaces per indentation level.
- Maximum line width: 100 characters.
- Block indent, not visual indent.
- Trailing comma in any multi-line comma-separated list.
- Zero or one blank line between items and statements.
- No trailing whitespace.
- One attribute per line; a single `#[derive(...)]` per item.
- Doc comments with `///` placed before attributes; `//!` only for crate or module docs.

Naming, following Rust conventions:

| Item | Case | Example |
| --- | --- | --- |
| Crates | `kebab-case` (package), `snake_case` (in code) | `estela-media-ffmpeg` |
| Modules, functions, variables | `snake_case` | `keyframe_before` |
| Types, traits, enum variants | `UpperCamelCase` | `ScreenSource`, `DecodeBackend::Hardware` |
| Constants and statics | `SCREAMING_SNAKE_CASE` | `PROJECT_SLOT_SIZE` |
| Lifetimes | short, lowercase | `'a` |

## 5. Readability

Readable code is the priority. Code should explain itself without comments.

- **Names say what things are.** No abbreviations except widely known ones (`id`, `url`, `gpu`, `io`, `fps`).
- **Small functions.** One responsibility each; aim for under 40 lines.
- **Flat control flow.** Use early returns and `?`; avoid nesting deeper than three levels.
- **No magic numbers.** Use named constants.
- **Enums over booleans** in public APIs: `Visibility::Hidden`, not `false`.
- **Strong types for units:** `Time` for nanoseconds, never a bare `i64`.
- **One concept per module.** A file that grows past ~400 lines is a signal to split it.
- **Explicit errors.** One `Error` enum per crate with `thiserror`; `anyhow` only in `estela-app` and `estela-cli`.
- **No `unwrap()` or `expect()`** outside tests and `main` setup.
- **No `todo!()` or `panic!()`** for unfinished work; return `Error::NotImplemented`.
- **Imports** grouped in this order, with one blank line between groups: `std`, external crates, workspace crates, `crate`/`super`.
- **Iterator chains** stay short; if a chain needs a comment, turn it into a named function.

## 6. Comments

- **Every comment is a single line.** No multi-line comments, no block comments (`/* */`), no comment paragraphs.
- This applies to `//`, `///` and `//!` alike.
- Maximum 80 characters, including the comment sigil.
- A comment is a complete sentence: Capital letter, final period.
- Comments explain **why**, never **what**. If a comment is needed to explain what the code does, rewrite the code.
- Longer explanations go in `ARCHITECTURE.md` or a relevant Markdown file in `wl/`, referenced by a one-line comment.
- Every `unsafe` block has a one-line `// SAFETY:` comment directly above it.

```rust
// Good: Explains why.
// WGC timestamps use QPC, so input events must use QPC too.
let timestamp = clock.now();

// Bad: Restates the code.
// Get the current time from the clock.
let timestamp = clock.now();
```

## 7. Architecture rules

These come from `ARCHITECTURE.md` and the phase plan. Do not break them.

- Dependencies point **down** only. `cargo xtask check-deps` enforces the allowed graph in `wl/xtask/dependencies.json`.
- Shared core crates do not depend on `estela-app` or platform crates, except `estela-engine` through target-specific dependencies.
- The UI (`estela-app`) never imports platform crates; it talks only to `estela-engine` and `estela-library`.
- `unsafe` is forbidden (`#![forbid(unsafe_code)]`) except in `estela-platform-*` and `estela-media-ffmpeg`.
- Time is `Time(i64)` in nanoseconds since recording start. Coordinates are physical source pixels.
- `estela-motion` uses `libm` math only, never `f64::sin` and similar, to keep results identical on both systems.
- `wgpu` must stay at the exact version used by the pinned `iced` version.
- The `.estela` file is the source of truth; the database is an index.
- FFmpeg is LGPL and dynamically linked. Never add GPL dependencies; `cargo deny` checks this.
- Rust is shared only between Windows and Linux. macOS has an independent Swift implementation and no Rust dependencies.

## 8. Database

- PostgreSQL migrations live in `wl/crates/estela-library/migrations/`.
- **Never edit a published migration.** Add a new numbered file instead.
- Every table belongs to the `library` schema. Tables and columns use `snake_case`; table names are plural.
- All queries use `sqlx::query!` or `sqlx::query_as!`. Run `cargo sqlx prepare` from `wl/` after changing a query and commit the `.sqlx/` files under `wl/`.

## 9. Branches

Never commit to `main` or `develop` directly. Every change starts from `develop`
in its own branch.

| Branch prefix | Use for | Commit type |
| --- | --- | --- |
| `features/` | New functionality | `feat` |
| `fixed/` | Bug fixes | `fixed` |
| `refactor/` | Code restructuring without behavior change | `refactor` |
| `docs/` | Documentation only | `docs` |

Branch names are lowercase `kebab-case` and start with the OS when the work is
OS-specific:

```
features/win-wgc-capture
features/linux-portal-cursor
fixed/linux-dmabuf-import-crash
refactor/core-timeline-history
docs/core-database-library-model
```

Use `win`, `linux` or `core` (shared code) after the prefix.
The `docs/` branch prefix does not imply a repository directory.

## 10. Commits

Format:

```
<type>(<scope>): <summary>
```

- `type`: `feat`, `fixed`, `refactor` or `docs`. No other types.
- `scope`: the crate without the `estela-` prefix, or `win`, `linux`, `db`, `ci`.
- `summary`: imperative mood, lowercase start, no final period, at most 72 characters.
- Optional body after a blank line, wrapped at 72 characters, explaining why.
- One logical change per commit. A commit must build and pass tests on its own.

```
feat(platform-win): capture display with windows graphics capture
fixed(format): keep previous slot when crc check fails
refactor(timeline): move snapping into its own module
docs(db): describe library tables and rebuild rules
```

Changes to CI, build files or dependencies use `refactor` unless they add a
capability (`feat`) or fix a breakage (`fixed`).

## 11. Pull requests

- **Never open a pull request against `main`.** Pull requests target `develop`.
- Only maintainers merge `develop` into `main`, for releases.
- One pull request per branch, scoped to one OS or to shared code, never both unless the task names both.
- The title follows the commit format.
- The description states what changed, why, which OS it affects and how it was tested.
- All checks in section 12 pass before requesting review.

## 12. Checks before every commit

Run from the Rust workspace in `wl/`:

```bash
cd wl
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
cargo xtask check-deps
cargo sqlx prepare --workspace --check
```

On an OS-specific task, also build the other OS target if possible (for example
with `cargo check --target`), to confirm nothing outside the task broke.

## 13. Stop and ask when

- The task needs changes in `mac/`, in the other OS, or in a shared crate during an OS-specific task.
- A change would break a rule in sections 3, 7 or 8.
- A new dependency, a dependency version change, or a new `unsafe` block is needed.
- A published migration or the `.estela` format specification would have to change.
- The requirement is ambiguous.
