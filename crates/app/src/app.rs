//! The egui/eframe clock view: flat dark theme, 24-hour digits, Japanese reading.
//! Responsive layout (scales with the window), kana + romaji helper lines,
//! keyboard shortcuts and labelled controls.

use crate::command::Command;
use chrono::{Datelike, Local, Timelike};
use eframe::egui;
use egui::{Color32, FontFamily, RichText};

const BG: Color32 = Color32::from_rgb(8, 8, 10);
const INK: Color32 = Color32::from_rgb(236, 238, 240);
const DIM: Color32 = Color32::from_rgb(150, 155, 162);
const ACCENT: Color32 = Color32::from_rgb(255, 212, 0);

/// Launch the desktop window.
///
/// # Errors
/// Returns any error from `eframe`/winit/wgpu while creating the window.
pub fn run() -> eframe::Result<()> {
    let (iw, ih, irgba) =
        crate::icons::rasterize(include_bytes!("../assets/icons/clock.svg"), 256, [255, 212, 0]);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([820.0, 780.0])
            .with_min_inner_size([460.0, 480.0])
            .with_app_id("lin-apuri-tsuru-clock")
            .with_title("lin-apuri-tsūru-clock")
            .with_icon(egui::IconData {
                rgba: irgba,
                width: iw,
                height: ih,
            }),
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
        egui_extras::install_image_loaders(&cc.egui_ctx);
        apply_flat_dark(&cc.egui_ctx);
        Self {
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
                let _ = (h, m);
            }
        }
    }

    /// Bottom control bar (mouse); the same actions have keyboard shortcuts.
    fn controls(&mut self, ctx: &egui::Context, h: u8, mi: u8) {
        egui::TopBottomPanel::bottom("controls")
            .frame(egui::Frame::new().fill(BG).inner_margin(8.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let sz = egui::vec2(15.0, 15.0);
                    let speak = egui::Button::image_and_text(
                        egui::Image::new(egui::include_image!("../assets/icons/speaker-high.svg"))
                            .tint(INK)
                            .fit_to_exact_size(sz),
                        "Speak",
                    );
                    if ui
                        .add(speak)
                        .on_hover_text("Speak the time in Japanese (Space)")
                        .clicked()
                    {
                        self.dispatch(Command::SpeakTime, ctx, h, mi);
                    }
                    let romaji = egui::Button::image_and_text(
                        egui::Image::new(egui::include_image!("../assets/icons/translate.svg"))
                            .tint(INK)
                            .fit_to_exact_size(sz),
                        "Romaji",
                    );
                    if ui
                        .add(romaji)
                        .on_hover_text("Show/hide the romaji helper (R)")
                        .clicked()
                    {
                        self.dispatch(Command::ToggleRomaji, ctx, h, mi);
                    }
                    let fmt = if self.fmt24 { "24h" } else { "12h" };
                    let format = egui::Button::image_and_text(
                        egui::Image::new(egui::include_image!("../assets/icons/clock-afternoon.svg"))
                            .tint(INK)
                            .fit_to_exact_size(sz),
                        fmt,
                    );
                    if ui.add(format).on_hover_text("Toggle 24h / 12h (F)").clicked() {
                        self.dispatch(Command::ToggleFormat, ctx, h, mi);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("VOICEVOX:No.7（アナウンス）").size(12.0).color(DIM))
                            .on_hover_text("Japanese voice — attribution required");
                    });
                });
            });
    }
}

impl eframe::App for ClockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
        let now = Local::now();
        let (y, mo, d) = (now.year(), now.month() as u8, now.day() as u8);
        let (h, mi) = (now.hour() as u8, now.minute() as u8);

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

        self.controls(ctx, h, mi);

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG).inner_margin(24.0))
            .show(ctx, |ui| {
                let s = (ui.available_height() / 720.0).clamp(0.5, 1.2);
                ui.vertical_centered(|ui| {
                    ui.add_space(6.0 * s);
                    label(ui, &jptime::weekday_en(y, mo, d).to_uppercase(), 62.0 * s, INK);
                    label(ui, &jptime::part_of_day(h).to_uppercase(), 32.0 * s, DIM);
                    ui.add_space(4.0 * s);
                    label(ui, &clock, 150.0 * s, INK);
                    label(ui, &format!("{d} {} {y}", month_en(mo)), 32.0 * s, INK);
                    ui.add_space(10.0 * s);
                    rule(ui, s);
                    ui.add_space(10.0 * s);
                    label(ui, &t.kanji, 54.0 * s, ACCENT);
                    label(ui, &t.kana, 26.0 * s, DIM);
                    if self.show_romaji {
                        label(ui, &t.romaji, 24.0 * s, DIM);
                    }
                    ui.add_space(4.0 * s);
                    label(ui, &dt.kanji, 32.0 * s, INK);
                    if self.show_romaji {
                        label(ui, &dt.romaji, 20.0 * s, DIM);
                    }
                });
            });
    }
}

fn label(ui: &mut egui::Ui, text: &str, size: f32, color: Color32) {
    ui.label(RichText::new(text).size(size).color(color).strong());
}

fn rule(ui: &mut egui::Ui, scale: f32) {
    let w = ui.available_width() * 0.55;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 3.0 * scale), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, ACCENT);
}

fn month_en(m: u8) -> &'static str {
    [
        "",
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ][m as usize]
}

/// Flat dark theme: no rounding, dark fills, light text.
fn apply_flat_dark(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = BG;
    v.window_fill = BG;
    v.extreme_bg_color = BG;
    v.override_text_color = Some(INK);
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = egui::CornerRadius::ZERO;
    }
    ctx.set_visuals(v);
}

/// Best-effort: install a system Japanese font so kanji/kana render.
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
            fonts.font_data.insert(
                "jp".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "jp".to_owned());
            fonts
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .push("jp".to_owned());
            ctx.set_fonts(fonts);
            return;
        }
    }
}
