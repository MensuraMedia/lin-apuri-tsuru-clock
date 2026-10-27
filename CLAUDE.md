# CLAUDE.md — lin-apuri-tsūru-clock

Native Japanese learning clock. Pure Rust, egui/eframe on wgpu, Cargo workspace.

## Build / test / run
- `cargo test -p jptime` — core reading engine (fast; the correctness tests).
- `cargo run -p linclock --no-default-features` — headless CLI clock.
- `cargo run -p linclock --no-default-features -- --once` — print one reading and exit.
- `cargo run -p linclock` — full flat-dark GUI (first build compiles eframe/wgpu).
- `cargo build --workspace` — everything.

## Layout & layering
- `crates/jptime` — pure, dependency-free readings (kanji/kana/romaji). No I/O, no UI. `#![forbid(unsafe_code)]`.
- `crates/app` (`linclock`) — the impure layer: system clock (chrono), GUI (eframe, feature `gui`), audio (rodio, feature `audio`). Depends on jptime; jptime never depends on it.
- `command.rs` — the unified `Command` registry (UI/keys/CLI dispatch one id).

## Conventions
- 24-hour time by default (Japanese convention). Flat dark theme.
- Japanese readings MUST be correct (irregular forms tested) — a wrong reading teaches wrong Japanese.
- Every change: append to `changelog.md` (ISO-8601, append-only) and, per release, `CHANGELOG.md`.
- One feature per module; keep GUI/audio feature-gated so the core builds light.
- Dual MIT OR Apache-2.0. VOICEVOX attribution shown in-app.

## Audio engine
VOICEVOX (speaker No.7) via the voicepack pipeline produces the clip bank in `assets/audio/`.
The app plays the clip for the current time; see docs/AUDIO.md.
