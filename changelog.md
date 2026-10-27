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
| 2026-10-27T17:55:00 | Settings panel (new settings.rs): theme presets + custom accent/text/bg colour pickers, brightness dimmer, ToggleSettings command (⚙ / S key) |
| 2026-10-27T18:00:00 | Background images: folder scan, Fixed/Slideshow/Random modes, Gaussian blur + dim overlay (image crate); cover-fit + cached textures |
| 2026-10-27T18:05:00 | Default background set to flat #141414; text-size presets (Compact/Standard/Large/Huge) on top of the responsive fit |
| 2026-10-27T18:30:00 | Voice reworked to LIVE VOICEVOX (audio.rs: ureq→:50021, audio_query→synthesis); no chime, no clip bank; per-mora timing returned |
| 2026-10-27T18:35:00 | Karaoke: yellow underline tracks the active mora on the time/date lines; a 🔊 speaker on each Japanese line (click to hear) |
| 2026-10-27T18:40:00 | Toolbar is icon-only (no text labels); removed the VOICEVOX label from the bar; Compact text size is now the default |
| 2026-10-27T18:45:00 | Settings moved from a floating window to an in-app page (← / S / Esc to leave) |
| 2026-10-27T19:05:00 | Voice fix: VOICEVOX speaker 7 (ずんだもん — childlike) → 30 (No.7「アナウンス」), the intended adult announcer voice |
| 2026-10-27T19:10:00 | 🔊 on every line (weekday→火曜日, part-of-day→朝/午後/夕方/夜, big time, English date, + kanji lines); date line now shows kana with the karaoke underline; taller default window |
