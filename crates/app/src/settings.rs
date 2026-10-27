//! User settings: themes/colours/brightness and a background-image system
//! (folder, fixed / slideshow / random, blur). GUI-only.

use eframe::egui::{self, Color32};
use std::path::PathBuf;

/// A flat colour theme. The app paints only from these four roles.
#[derive(Clone, Copy, PartialEq)]
pub struct Theme {
    pub bg: Color32,
    pub ink: Color32,
    pub dim: Color32,
    pub accent: Color32,
}

impl Theme {
    pub const PRESETS: [(&'static str, Theme); 4] = [
        (
            "Flat Dark",
            Theme {
                bg: Color32::from_rgb(20, 20, 20),
                ink: Color32::from_rgb(236, 238, 240),
                dim: Color32::from_rgb(150, 155, 162),
                accent: Color32::from_rgb(255, 212, 0),
            },
        ),
        (
            "Midnight",
            Theme {
                bg: Color32::from_rgb(10, 14, 22),
                ink: Color32::from_rgb(224, 232, 240),
                dim: Color32::from_rgb(120, 140, 160),
                accent: Color32::from_rgb(90, 180, 255),
            },
        ),
        (
            "Paper (light)",
            Theme {
                bg: Color32::from_rgb(244, 244, 246),
                ink: Color32::from_rgb(20, 22, 26),
                dim: Color32::from_rgb(110, 114, 120),
                accent: Color32::from_rgb(205, 115, 0),
            },
        ),
        (
            "High Contrast",
            Theme {
                bg: Color32::BLACK,
                ink: Color32::WHITE,
                dim: Color32::from_rgb(205, 205, 205),
                accent: Color32::from_rgb(255, 235, 0),
            },
        ),
    ];

    pub fn default_dark() -> Theme {
        Self::PRESETS[0].1
    }

    /// Is this a light background (text-over-image needs a lighter scrim)?
    pub fn is_light(&self) -> bool {
        let c = self.bg;
        0.2126 * c.r() as f32 + 0.7152 * c.g() as f32 + 0.0722 * c.b() as f32 > 140.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BgMode {
    None,
    Fixed,
    Slideshow,
    Random,
}

impl BgMode {
    pub fn label(self) -> &'static str {
        match self {
            BgMode::None => "None",
            BgMode::Fixed => "Fixed",
            BgMode::Slideshow => "Slideshow",
            BgMode::Random => "Random",
        }
    }
}

/// Background-image state: a folder of images shown behind the clock.
pub struct Background {
    pub dir: String,
    pub images: Vec<PathBuf>,
    pub mode: BgMode,
    pub blur: f32,
    pub interval_s: f32,
    pub scrim: f32,
    pub idx: usize,
    pub status: String,
    last_switch: f64,
    cache: Option<(PathBuf, u32, egui::TextureHandle)>,
}

impl Default for Background {
    fn default() -> Self {
        Self {
            dir: String::new(),
            images: Vec::new(),
            mode: BgMode::None,
            blur: 8.0,
            interval_s: 20.0,
            scrim: 0.55,
            idx: 0,
            status: String::new(),
            last_switch: 0.0,
            cache: None,
        }
    }
}

impl Background {
    /// Scan `dir` for image files (png/jpg/jpeg/webp), sorted.
    pub fn scan(&mut self) {
        self.images.clear();
        self.idx = 0;
        self.cache = None;
        let dir = PathBuf::from(shellexpand(&self.dir));
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for e in rd.flatten() {
                let p = e.path();
                if matches!(
                    p.extension()
                        .and_then(|x| x.to_str())
                        .map(|s| s.to_ascii_lowercase())
                        .as_deref(),
                    Some("png" | "jpg" | "jpeg" | "webp")
                ) {
                    self.images.push(p);
                }
            }
            self.images.sort();
        }
        self.status = if self.images.is_empty() {
            format!("no images in {}", dir.display())
        } else {
            format!("{} image(s)", self.images.len())
        };
        if self.images.is_empty() {
            self.mode = BgMode::None;
        }
    }

    /// Advance the shown image for slideshow/random when the interval elapses.
    pub fn tick(&mut self, now: f64) {
        if self.images.len() < 2 {
            return;
        }
        let due = now - self.last_switch >= self.interval_s as f64;
        match self.mode {
            BgMode::Slideshow if due => {
                self.idx = (self.idx + 1) % self.images.len();
                self.last_switch = now;
            }
            BgMode::Random if due => {
                // cheap deterministic-ish shuffle step from the clock
                let n = self.images.len();
                let step = ((now as usize).wrapping_mul(2_654_435_761) >> 13) % n;
                self.idx = (self.idx + 1 + step) % n;
                self.last_switch = now;
            }
            _ => {}
        }
    }

    fn current_path(&self) -> Option<PathBuf> {
        if self.mode == BgMode::None {
            return None;
        }
        self.images
            .get(self.idx.min(self.images.len().saturating_sub(1)))
            .cloned()
    }

    /// The texture for the current image at the current blur (cached).
    pub fn texture(&mut self, ctx: &egui::Context) -> Option<egui::TextureHandle> {
        let path = self.current_path()?;
        let blur_key = self.blur.round() as u32;
        if let Some((cp, cb, tex)) = &self.cache
            && *cp == path
            && *cb == blur_key
        {
            return Some(tex.clone());
        }
        let img = load_blurred(&path, self.blur)?;
        let tex = ctx.load_texture("background", img, egui::TextureOptions::LINEAR);
        self.cache = Some((path, blur_key, tex.clone()));
        Some(tex)
    }
}

/// Decode an image, optionally Gaussian-blur it, as an egui `ColorImage`.
fn load_blurred(path: &std::path::Path, blur: f32) -> Option<egui::ColorImage> {
    let decoded = image::open(path).ok()?.to_rgba8();
    // blur on a downscaled copy when large — fast, and the result is soft anyway
    let (w, h) = decoded.dimensions();
    let max = 1600u32;
    let decoded = if w.max(h) > max {
        let s = max as f32 / w.max(h) as f32;
        image::imageops::resize(
            &decoded,
            (w as f32 * s) as u32,
            (h as f32 * s) as u32,
            image::imageops::FilterType::Triangle,
        )
    } else {
        decoded
    };
    let buf = if blur >= 1.0 {
        image::imageops::blur(&decoded, blur)
    } else {
        decoded
    };
    let (w, h) = buf.dimensions();
    Some(egui::ColorImage::from_rgba_unmultiplied(
        [w as usize, h as usize],
        buf.as_raw(),
    ))
}

/// Expand a leading `~` to the home directory.
fn shellexpand(s: &str) -> String {
    if let Some(rest) = s.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return format!("{home}/{rest}");
    }
    s.to_string()
}
