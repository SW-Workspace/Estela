# Development

Windows and Linux use the Rust toolchain pinned in `wl/rust-toolchain.toml`.
Install it with rustup, including rustfmt and Clippy. Run checks from `wl/`:

```sh
cd wl
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
cargo xtask check-deps
cargo xtask check-sqlx
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
```

SQLx integration and its offline metadata are pending. `cargo xtask check-sqlx`
runs `cargo sqlx prepare --workspace --check` once a workspace crate depends on
SQLx. Until then, CI explicitly reports that the check is not applicable. Do not
add SQLx or change dependencies as part of an unrelated task.

Follow [AGENTS.md](AGENTS.md). Documentation, comments and diagnostics use English.
Executable UI messages live in `wl/locales/en/`, with Spanish translations in
`wl/locales/es/`. `LC_ALL`, `LC_MESSAGES` and `LANG` select the message language,
in that order; English is the fallback.

Do not modify published migrations or include secrets, PostgreSQL clusters or
third-party binaries in Git. Use Git LFS for PNG and `.estela` fixtures.

macOS uses Swift only and is outside this workflow. Reference documents live in
`wl/reference/`; there is no root `docs/` directory.
