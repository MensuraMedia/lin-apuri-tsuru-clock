//! Spoken Japanese via the local **VOICEVOX** engine (speaker No.7「アナウンス」).
//! No chime — the text is synthesised live. `speak` returns per-mora timing so the
//! UI can underline the exact sound being pronounced (a karaoke cursor), and plays
//! the audio through the OS player. Any minute/date works without a pre-rendered bank.

/// One mora (smallest Japanese sound unit) and when it sounds, in seconds from
/// the start of playback. The timing types are always compiled; the engine that
/// fills them is behind the `audio` feature.
#[cfg(feature = "audio")]
#[derive(Clone)]
pub struct Mora {
    pub kana: String,
    pub start: f32,
    pub end: f32,
}

/// A spoken line: the mora timeline and total length in seconds.
#[cfg(feature = "audio")]
#[derive(Clone)]
pub struct Utterance {
    pub moras: Vec<Mora>,
    pub total: f32,
}

#[cfg(feature = "audio")]
pub use engine::{Speaker, warm_up};

#[cfg(feature = "audio")]
mod engine {
    use super::{Mora, Utterance};
    use std::io::{Read, Write};
    use std::path::Path;
    use std::time::Duration;

    const BASE: &str = "http://127.0.0.1:50021";
    const SPEAKER: u32 = 30;

    /// A handle to the local VOICEVOX engine.
    pub struct Speaker;

    impl Speaker {
        /// `Some` if the VOICEVOX engine answers on :50021.
        pub fn detect() -> Option<Speaker> {
            ureq::get(&format!("{BASE}/version"))
                .timeout(Duration::from_millis(500))
                .call()
                .ok()
                .map(|_| Speaker)
        }

        /// Query timing, synthesise, and start playback. Returns the mora
        /// timeline; the caller starts its clock when this returns (playback
        /// has just begun).
        pub fn speak(&self, text: &str) -> Option<Utterance> {
            let query: serde_json::Value = ureq::post(&format!("{BASE}/audio_query"))
                .query("speaker", &SPEAKER.to_string())
                .query("text", text)
                .timeout(Duration::from_secs(5))
                .call()
                .ok()?
                .into_json()
                .ok()?;

            let speed = query["speedScale"].as_f64().unwrap_or(1.0).max(0.1) as f32;
            let pre = query["prePhonemeLength"].as_f64().unwrap_or(0.1) as f32;
            let mut moras = Vec::new();
            let mut t = pre / speed;
            for ap in query["accent_phrases"].as_array()? {
                if let Some(list) = ap["moras"].as_array() {
                    for m in list {
                        let c = m["consonant_length"].as_f64().unwrap_or(0.0) as f32;
                        let v = m["vowel_length"].as_f64().unwrap_or(0.0) as f32;
                        let dur = ((c + v) / speed).max(0.0);
                        let kana = kata_to_hira(m["text"].as_str().unwrap_or(""));
                        moras.push(Mora {
                            kana,
                            start: t,
                            end: t + dur,
                        });
                        t += dur;
                    }
                }
                if let Some(p) = ap.get("pause_mora").and_then(|p| p.as_object()) {
                    t += p.get("vowel_length").and_then(|x| x.as_f64()).unwrap_or(0.0) as f32 / speed;
                }
            }
            t += query["postPhonemeLength"].as_f64().unwrap_or(0.1) as f32 / speed;

            let wav = ureq::post(&format!("{BASE}/synthesis"))
                .query("speaker", &SPEAKER.to_string())
                .timeout(Duration::from_secs(20))
                .send_json(query)
                .ok()?;
            let mut bytes = Vec::new();
            wav.into_reader().read_to_end(&mut bytes).ok()?;
            let path = std::env::temp_dir().join("lin-apuri-tsuru-clock-say.wav");
            std::fs::File::create(&path).ok()?.write_all(&bytes).ok()?;
            play(&path);

            Some(Utterance { moras, total: t })
        }
    }

    /// Preload speaker 30's model in the background so the first click is snappy
    /// (VOICEVOX lazy-loads a voice on first use, which can take seconds).
    pub fn warm_up() {
        std::thread::spawn(|| {
            let _ = ureq::post(&format!("{BASE}/initialize_speaker"))
                .query("speaker", &SPEAKER.to_string())
                .timeout(Duration::from_secs(30))
                .call();
        });
    }

    /// Spawn the OS audio player on `path` (detached; we don't wait).
    fn play(path: &Path) {
        for p in ["paplay", "pw-play", "aplay", "afplay"] {
            if std::process::Command::new(p).arg(path).spawn().is_ok() {
                return;
            }
        }
    }

    /// VOICEVOX returns katakana; show hiragana to match the rest of the UI.
    fn kata_to_hira(s: &str) -> String {
        s.chars()
            .map(|c| {
                if ('\u{30A1}'..='\u{30F6}').contains(&c) {
                    char::from_u32(c as u32 - 0x60).unwrap_or(c)
                } else {
                    c
                }
            })
            .collect()
    }
}
