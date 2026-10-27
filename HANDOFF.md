# Handoff — lin-apuri-tsūru-clock

A working handoff for the next engineer or agent. Pairs with `README.md` (what & why),
`docs/ARCHITECTURE.md` (layering), `docs/AUDIO.md` (voice), `CLAUDE.md` (conventions) and
the append-only `changelog.md`.

## 1. State (2026-10-27)

- **Builds clean**: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace` all pass. CI runs the same (`.github/workflows/ci.yml`).
- **Runs**: native GUI verified (screenshot in `assets/screenshots/clock.png`); live VOICEVOX voice + karaoke underline, background images, in-app settings page, Compact default, and text-size all verified; headless `--once`/`--no-default-features` verified.
- **Time source**: the local OS clock/timezone via `chrono::Local::now()`. **Voice** needs the local **VOICEVOX** engine on `:50021` (speaker No.7); without it the clock runs, the 🔊 just no-ops.

## 2. Layout (where things live)

| Path | What |
|---|---|
| `crates/jptime/` | Pure reading engine (kanji/kana/romaji), `#![forbid(unsafe_code)]`, unit-tested. No I/O. |
| `crates/app/src/main.rs` | Entry point; CLI (`--once`, `--cmd`, `--list-commands`) and GUI dispatch. |
| `crates/app/src/app.rs` | `ClockApp` (eframe): flat dark theme, 24h layout, responsive scale, window icon. |
| `crates/app/src/command.rs` | `Command` registry — the single dispatch for GUI/keys/CLI. |
| `crates/app/src/audio.rs` | VOICEVOX `Speaker` — live synthesis (:50021) + per-mora timing; plays via the OS player. |
| `crates/app/src/icons.rs` | Rasterises a bundled SVG to RGBA for the window/taskbar icon (resvg). |
| `crates/app/src/settings.rs` | Theme presets, background-image system (folder, modes, blur), text-size. |
| `crates/app/assets/icons/` | Bundled Phosphor icons (MIT) + `ATTRIBUTION.md`. |
| `assets/audio/` | Hourly VOICEVOX clips (git-ignored; regenerate — see §5). |
| `docs/architecture.svg` | Architecture diagram (diagram-design skill). |
| `packaging/` | `.desktop`, app icon, `install.sh`/`uninstall.sh`, `build-packages.sh`. |

## 3. Build / run

```bash
cargo test -p jptime                          # fast, no GUI deps
cargo run  -p linclock                         # full GUI (first build compiles eframe/wgpu)
cargo run  -p linclock --no-default-features -- --once   # headless one-shot
bash packaging/install.sh                      # user-level menu/panel/Alt-Tab launcher
```

## 4. Conventions

- **Layering is law**: `jptime` never imports `app`; impurity (clock, UI, audio) stays in `linclock`, behind `gui`/`audio` features.
- **One dispatch**: add a UI action by adding a `Command` variant + `id()` — never a one-off handler.
- **Readings are tested**: any change to `jptime` must keep every irregular-form test green (a wrong reading teaches wrong Japanese).
- **Log every change** in `changelog.md` (ISO-8601, append-only).

## 5. Known gaps / next steps (priority order)

1. **Resident systray icon** — the panel presence today is the `.desktop` launcher; a true tray needs the `tray-icon` crate wired into eframe's event loop (not yet done).
3. **WASM build** (`trunk`) and on-the-hour announcement — see README roadmap.
4. **Pitch-accent overlay** — VOICEVOX already returns per-mora accent; the karaoke renderer is the natural place to show it.

## 6. Gotchas

- `resvg` is built with `default-features = false` (no text rendering) because the app only rasterises the icon (no text). The README diagram's text renders in the browser/GitHub, not via the app.
- First GUI build compiles eframe/wgpu — minutes. Subsequent builds are cached.
- The GUI needs a display + GPU; without one the app prints the reading once and exits (graceful fallback).
