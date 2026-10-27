//! lin-apuri-tsūru-clock: a native Japanese learning clock. See README.md.
//!
//! GUI by default (feature `gui`); `--once` or a no-GUI build prints one reading.

mod command;
#[cfg(feature = "audio")]
mod audio;
#[cfg(feature = "gui")]
mod app;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--list-commands") {
        for c in [command::Command::SpeakTime, command::Command::ToggleRomaji,
                  command::Command::ToggleFormat, command::Command::Quit] {
            println!("{}", c.id());
        }
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--cmd") {
        match args.get(i + 1).and_then(|id| command::Command::from_id(id)) {
            Some(command::Command::SpeakTime) => {
                run_speak();
                return;
            }
            Some(c) => {
                eprintln!("command '{}' only applies to the running GUI", c.id());
                return;
            }
            None => {
                eprintln!("unknown command; try --list-commands");
                return;
            }
        }
    }

    #[cfg(feature = "gui")]
    {
        let once = std::env::args().any(|a| a == "--once");
        if !once {
            match app::run() {
                Ok(()) => return,
                Err(e) => eprintln!("GUI unavailable ({e}); printing once instead:"),
            }
        }
    }
    print_once();
}

/// Speak the current time (headless), if the audio engine and a clip are present.
fn run_speak() {
    #[cfg(feature = "audio")]
    {
        use chrono::{Local, Timelike};
        let now = Local::now();
        if let Some(p) = audio::Player::new("assets/audio") {
            if p.speak_time(now.hour() as u8, now.minute() as u8) {
                return;
            }
        }
        eprintln!("no audio player or no clip for this time (see docs/AUDIO.md)");
    }
    #[cfg(not(feature = "audio"))]
    eprintln!("built without the 'audio' feature");
}

/// The clock as one block of text (headless / scripting / fallback).
fn print_once() {
    use chrono::{Datelike, Local, Timelike};
    let now = Local::now();
    let (y, m, d) = (now.year(), now.month() as u8, now.day() as u8);
    let (h, mi) = (now.hour() as u8, now.minute() as u8);
    let t = jptime::time_reading(h, mi);
    let dt = jptime::date_reading(y, m, d);
    println!("{:02}:{:02}   {}   ({})", h, mi, t.kanji, t.romaji);
    println!(
        "{} {}   {}   ({})",
        jptime::weekday_en(y, m, d),
        jptime::part_of_day(h),
        dt.kanji,
        dt.romaji
    );
}
