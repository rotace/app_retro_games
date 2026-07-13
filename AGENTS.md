<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan:
specs/003-debian-perlin-noise-drawing/plan.md
<!-- SPECKIT END -->

## Cursor Cloud specific instructions

Rust Cargo workspace (`stable` toolchain pinned in `rust-toolchain.toml`) with three crates:

- `retro_core` — library with Perlin noise generation + render traits.
- `runner-debian` — CLI that draws to the Linux framebuffer (`/dev/fb0`).
- `runner-wslg` — GUI (minifb) "simulation" runner.

Standard commands (workspace root): `cargo build`, `cargo test`, `cargo clippy --all-targets`, `cargo fmt --all`. See `specs/003-debian-perlin-noise-drawing/quickstart.md` for feature-level usage.

Non-obvious caveats:

- `runner-debian` cannot run in this cloud VM: it requires `/dev/fb0`, root, and VT switching (opens `/dev/tty7`), and `main.rs` uses `.expect()` on those, so it will panic here. It is meant for real old-Debian hardware only. It still builds/tests normally.
- `runner-wslg`'s `main.rs` is just a `Hello, World!` stub; the real rendering (`WindowTarget`) lives in `src/lib.rs` and is exercised by the integration test `cargo test -p runner-wslg`, which opens a GUI window against the X display (`DISPLAY=:1` is available in the cloud VM). To see it visually, drive `WindowTarget` in a loop (e.g. a temporary `examples/` binary).
- Cross-compiling `runner-debian` for its real target (`cargo build -p runner-debian --target i686-unknown-linux-musl`) needs `musl-gcc` (the linker set in `.cargo/config.toml`), provided by the `musl-tools` system package. The default host build does not need it.
