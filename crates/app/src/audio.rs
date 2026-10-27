//! Spoken-time playback via the operating system's audio player (no build-time
//! dependencies, uses tools already installed). The audio *content* is produced
//! by the project's audio engine (VOICEVOX via voicepack) as WAV clips under
//! `assets/audio/`; this module just plays the clip for a given time.
//! Best-effort: no player or clip = silence, never a panic.
#![cfg(feature = "audio")]

use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Player {
    dir: PathBuf,
    cmd: &'static str,
}

impl Player {
    /// Find an OS audio player and the clip directory. `None` if none is found.
    pub fn new(dir: impl AsRef<Path>) -> Option<Player> {
        detect_player().map(|cmd| Player { dir: dir.as_ref().to_path_buf(), cmd })
    }

    /// Play `assets/audio/time_HHMM.wav` for the time rounded down to the nearest
    /// available step. Returns false if no matching clip was found.
    pub fn speak_time(&self, hour: u8, minute: u8) -> bool {
        for step in [1u8, 5, 10, 15, 20, 30, 60] {
            let m = (minute / step) * step;
            let path = self.dir.join(format!("time_{hour:02}{m:02}.wav"));
            if path.is_file() {
                return Command::new(self.cmd).arg(&path).spawn().is_ok();
            }
        }
        false
    }
}

fn detect_player() -> Option<&'static str> {
    #[cfg(target_os = "macos")]
    let candidates: &[&str] = &["afplay"];
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let candidates: &[&str] = &["paplay", "pw-play", "aplay"];
    #[cfg(target_os = "windows")]
    let candidates: &[&str] = &[]; // Windows playback: a roadmap item

    candidates.iter().copied().find(|c| which(c))
}

fn which(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {cmd}"))
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
