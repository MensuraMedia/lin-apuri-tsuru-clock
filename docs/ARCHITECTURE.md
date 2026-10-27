# Architecture

Two layers, strict one-way dependency.

```
linclock (app, impure)   egui/eframe/wgpu window · chrono clock · OS audio player
      │  depends on
jptime (core, pure)      kanji/kana/romaji readings · no I/O · no deps · no unsafe
```

## jptime (core)
Deterministic functions `time_reading`, `date_reading`, `weekday_en`, `part_of_day`.
Input numbers, output `Reading { kanji, kana, romaji }`. The irregular readings are
encoded in explicit tables and unit-tested. No system clock, no files, no UI — so it
is reusable from the GUI, the CLI, a web build, or another project.

## linclock (app)
Everything impure, feature-gated:
- `gui` (default): `app.rs` renders the flat-dark, 24-hour clock with the Japanese
  lines; repaints every ~0.5 s; installs a system CJK font best-effort.
- `audio` (default): `audio.rs` plays a WAV clip for the current time via the OS
  player (`paplay`/`pw-play`/`aplay`, or `afplay` on macOS) — no build-time deps.
- `command.rs`: the unified `Command` registry. GUI buttons, keyboard shortcuts and
  the CLI (`--cmd <id>`, `--list-commands`) all dispatch the same ids — the hook for
  headless / agent / future JSON or MCP control.

## Build profiles
`--no-default-features` gives a dependency-light CLI (jptime + chrono) for scripting
and CI. The default build adds the GUI and audio.
