//! The egui/eframe clock view: flat dark theme, 24-hour digits, Japanese reading.
#![cfg(feature = "gui")]

use crate::command::Command;
use chrono::{Datelike, Local, Timelike};
use eframe::egui;
use egui::{Color32, FontFamily, RichText};

const BG: Color32 = Color32::from_rgb(8, 8, 10);
const INK: Color32 = Color32::from_rgb(236, 238, 240);
const DIM: Color32 = Color32::from_rgb(150, 155, 162);
const ACCENT: Color32 = Color32::from_rgb(255, 212, 0);

pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([820.0, 620.0])
            .with_min_inner_size([480.0, 360.0])
            .with_title("lin-apuri-tsūru-clock"),
        ..Default::default()
    };
    eframe::run_native(
        "lin-apuri-tsūru-clock",
        options,
        Box::new(|cc| Ok(Box::new(ClockApp::new(cc)))),
    )
}

struct ClockApp {
    show_romaji: bool,
    fmt24: bool,
    #[cfg(feature = "audio")]
    player: Option<crate::audio::Player>,
}

impl ClockApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_japanese_font(&cc.egui_ctx);
        apply_flat_dark(&cc.egui_ctx);
        ClockApp {
            show_romaji: true,
            fmt24: true,
            #[cfg(feature = "audio")]
            player: crate::audio::Player::new("assets/audio"),
        }
    }

    fn dispatch(&mut self, cmd: Command, ctx: &egui::Context, h: u8, m: u8) {
        match cmd {
            Command::ToggleRomaji => self.show_romaji = !self.show_romaji,
            Command::ToggleFormat => self.fmt24 = !self.fmt24,
            Command::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Command::SpeakTime => {
                #[cfg(feature = "audio")]
                if let Some(p) = &self.player {
                    let _ = p.speak_time(h, m);
                }
                #[cfg(not(feature = "audio"))]
                {
                    let _ = (h, m);
                }
            }
        }
    }
}

impl eframe::App for ClockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
        let now = Local::now();
        let (y, mo, d) = (now.year(), now.month() as u8, now.day() as u8);
        let (h, mi) = (now.hour() as u8, now.minute() as u8);

        // keyboard shortcuts -> commands
        for (key, cmd) in [
            (egui::Key::Space, Command::SpeakTime),
            (egui::Key::R, Command::ToggleRomaji),
            (egui::Key::F, Command::ToggleFormat),
            (egui::Key::Escape, Command::Quit),
        ] {
            if ctx.input(|i| i.key_pressed(key)) {
                self.dispatch(cmd, ctx, h, mi);
            }
        }

        let t = jptime::time_reading(h, mi);
        let dt = jptime::date_reading(y, mo, d);
        let clock = if self.fmt24 {
            format!("{h:02}:{mi:02}")
        } else {
            let h12 = if h % 12 == 0 { 12 } else { h % 12 };
            format!("{h12}:{mi:02} {}", if h < 12 { "AM" } else { "PM" })
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG).inner_margin(24.0))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    label(ui, &jptime::weekday_en(y, mo, d).to_uppercase(), 64.0, INK);
                    label(ui, &jptime::part_of_day(h).to_uppercase(), 34.0, DIM);
                    ui.add_space(6.0);
                    label(ui, &clock, 150.0, INK);
                    label(ui, &format!("{} {} {}", d, month_en(mo), y), 34.0, INK);
                    ui.add_space(10.0);
                    rule(ui);
                    ui.add_space(10.0);
                    label(ui, &t.kanji, 56.0, ACCENT);
                    if self.show_romaji {
                        label(ui, &t.romaji, 30.0, DIM);
                    }
                    label(ui, &dt.kanji, 34.0, INK);
                    if self.show_romaji {
                        label(ui, &dt.romaji, 22.0, DIM);
                    }
                });

                // bottom controls (mouse users); keys also work
                egui::TopBottomPanel::bottom("controls")
                    .frame(egui::Frame::new().fill(BG))
                    .show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("🔊 Speak  (space)").clicked() {
                                self.dispatch(Command::SpeakTime, ctx, h, mi);
                            }
                            if ui.button("あ Romaji  (r)").clicked() {
                                self.dispatch(Command::ToggleRomaji, ctx, h, mi);
                            }
                            if ui.button("24h / 12h  (f)").clicked() {
                                self.dispatch(Command::ToggleFormat, ctx, h, mi);
                            }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new("VOICEVOX:No.7（アナウンス）").size(12.0).color(DIM));
                            });
                        });
                    });
            });
    }
}

fn label(ui: &mut egui::Ui, text: &str, size: f32, color: Color32) {
    ui.label(RichText::new(text).size(size).color(color).strong());
}

fn rule(ui: &mut egui::Ui) {
    let w = ui.available_width() * 0.6;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 3.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, ACCENT);
}

fn month_en(m: u8) -> &'static str {
    ["", "January", "February", "March", "April", "May", "June", "July", "August", "September",
     "October", "November", "December"][m as usize]
}

/// Flat dark theme: no rounding, dark fills, light text.
fn apply_flat_dark(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = BG;
    v.window_fill = BG;
    v.extreme_bg_color = BG;
    v.override_text_color = Some(INK);
    for w in [
        &mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered,
        &mut v.widgets.active, &mut v.widgets.open,
    ] {
        w.corner_radius = egui::CornerRadius::ZERO;
    }
    ctx.set_visuals(v);
}

/// Best-effort: install a system Japanese font so kanji/kana render.
/// If none is found the app still runs (romaji always renders).
fn install_japanese_font(ctx: &egui::Context) {
    const CANDIDATES: &[&str] = &[
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/fonts-japanese-gothic.ttf",
        "/usr/share/fonts/opentype/ipafont-gothic/ipag.ttf",
    ];
    for path in CANDIDATES {
        if let Ok(bytes) = std::fs::read(path) {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert("jp".to_owned(), std::sync::Arc::new(egui::FontData::from_owned(bytes)));
            fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "jp".to_owned());
            fonts.families.entry(FontFamily::Monospace).or_default().push("jp".to_owned());
            ctx.set_fonts(fonts);
            return;
        }
    }
}
