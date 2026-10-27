# Project Change Log

> Append-only, ISO-8601, independent of git (per MensuraMedia universal instruction set).

| Date-Time | Change Description |
|-----------|-------------------|
| 2026-10-27T13:02:01 | Project created: Rust workspace (edition 2024), jptime core crate (readings, 3 tests passing), linclock CLI verified (24h + Japanese) |
| 2026-10-27T13:02:01 | linclock GUI (egui/eframe/wgpu, flat dark, 24h, Japanese, audio) written; unified Command registry; audio module (rodio) |
| 2026-10-27T13:02:01 | Advanced README, CLAUDE.md, dual MIT/Apache licences, docs (ARCHITECTURE, AUDIO) |
| 2026-10-27T17:00:00 | audio.rs reworked to spawn the OS player (paplay/pw-play/aplay/afplay) instead of linking rodio/ALSA — no system build deps |
| 2026-10-27T17:10:00 | Quality gates: cargo fmt + clippy (`-D warnings`) clean; workspace lints; rust-toolchain.toml; GitHub Actions CI (fmt·clippy·test·build) |
| 2026-10-27T17:20:00 | UI/UX: responsive type scale, kana helper line, tooltips on all controls, taller default window; time confirmed from OS local clock (chrono::Local) |
| 2026-10-27T17:30:00 | Iconography: bundled Phosphor (MIT) icon set; window/taskbar/Alt-Tab icon via resvg; toolbar icons via egui_extras SVG loader |
| 2026-10-27T17:35:00 | Diagram: docs/architecture.svg authored per the diagram-design skill (one accent, hairlines, 4px grid, a11y), verified render |
| 2026-10-27T17:40:00 | Packaging: .desktop launcher (+StartupWMClass), branded app icon, user-level install/uninstall scripts, AppImage/.deb builder |
| 2026-10-27T17:45:00 | Real screenshot captured (assets/screenshots/clock.png); HANDOFF.md added; README refreshed (diagram, screenshot, launchers, tooling) |
