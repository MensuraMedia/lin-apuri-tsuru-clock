//! Unified command registry. Every UI action, shortcut and CLI call dispatches
//! by the same `Command`, so the app is drivable headless (an agent/CLI uses the
//! same IDs as the GUI). A small, universal pattern meant to be reused.

/// A thing the app can be asked to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Speak the current time (and date) in Japanese.
    SpeakTime,
    /// Toggle the romaji helper line.
    ToggleRomaji,
    /// Toggle 24-hour / 12-hour digits (default 24-hour, as in Japan).
    ToggleFormat,
    /// Quit the application.
    Quit,
}

impl Command {
    /// Stable string id (for CLI, config, a future JSON/MCP control channel).
    pub const fn id(self) -> &'static str {
        match self {
            Command::SpeakTime => "speak_time",
            Command::ToggleRomaji => "toggle_romaji",
            Command::ToggleFormat => "toggle_format",
            Command::Quit => "quit",
        }
    }

    /// Parse a command from its id.
    pub fn from_id(id: &str) -> Option<Command> {
        [
            Command::SpeakTime,
            Command::ToggleRomaji,
            Command::ToggleFormat,
            Command::Quit,
        ]
        .into_iter()
        .find(|c| c.id() == id)
    }
}
