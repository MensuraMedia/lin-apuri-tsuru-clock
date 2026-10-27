> **Note:** the desktop app now synthesises live via VOICEVOX (`audio.rs`, no chime, no clip bank). This document describes the legacy `voicepack` clip-bank generator (still used by the clock-mod firmware project).

# Audio

The app **speaks** the time in Japanese. Playback and content are separated.

## Playback (in this app, no dependencies)
`crates/app/src/audio.rs` plays `assets/audio/time_HHMM.wav` for the current time
by spawning the operating system's audio player (`paplay`, `pw-play`, `aplay`, or
`afplay`). No build-time audio library is required, so the app builds anywhere a
Rust toolchain exists. Missing player or clip → silence, never a crash.

## Content (the engine)
The spoken clips are produced by **VOICEVOX** — a Japanese-native TTS engine
(speaker No.7「アナウンス」) — driven by the `voicepack` pipeline from
[clock-mod](https://github.com/MensuraMedia/clock-mod):

1. `jptime` gives the kanji reading of each time (e.g. `午後3時`).
2. VOICEVOX synthesises it with correct pitch accent, at a measured speaking rate.
3. A chime is prepended; the clip is validated (decoded peak + duration) so a silent
   render can never ship.

Generate / refresh the bank (VOICEVOX running on `127.0.0.1:50021`):

```bash
# in clock-mod:
python3 -m voicepack timeclock --step 60 --out /tmp/tc
# convert to WAV for this app's zero-dep player:
for f in /tmp/tc/time_*.mp3; do
  ffmpeg -v error -y -i "$f" "assets/audio/$(basename "${f%.mp3}").wav"; done
```

## Attribution (required)
VOICEVOX requires visible credit. The app shows `VOICEVOX:No.7（アナウンス）`; keep it.
