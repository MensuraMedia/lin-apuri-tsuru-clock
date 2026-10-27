//! The egui/eframe clock view: themeable flat UI, 24-hour digits, Japanese
//! reading, optional background images, brightness, and a settings panel.

use crate::command::Command;
use crate::settings::{Background, BgMode, Theme};
use chrono::{Datelike, Local, Timelike};
use eframe::egui;
use egui::{Color32, FontFamily, RichText};

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
    theme: Theme,
    brightness: f32,
    text_scale: f32,
    settings_open: bool,
    background: Background,
    #[cfg(feature = "audio")]
    player: Option<crate::audio::Player>,
}

impl ClockApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_japanese_font(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Self {
            show_romaji: true,
            fmt24: true,
            theme: Theme::default_dark(),
            brightness: 1.0,
            text_scale: 1.0,
            settings_open: false,
            background: Background::default(),
            #[cfg(feature = "audio")]
            player: crate::audio::Player::new("assets/audio"),
        }
    }

    fn dispatch(&mut self, cmd: Command, ctx: &egui::Context, h: u8, m: u8) {
        match cmd {
            Command::ToggleRomaji => self.show_romaji = !self.show_romaji,
            Command::ToggleFormat => self.fmt24 = !self.fmt24,
            Command::ToggleSettings => self.settings_open = !self.settings_open,
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
        let (ink, dim, bg) = (self.theme.ink, self.theme.dim, self.theme.bg);
        egui::TopBottomPanel::bottom("controls")
            .frame(egui::Frame::new().fill(bg).inner_margin(8.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let sz = egui::vec2(15.0, 15.0);
                    let speak = egui::Button::image_and_text(
                        egui::Image::new(egui::include_image!("../assets/icons/speaker-high.svg"))
                            .tint(ink)
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
                            .tint(ink)
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
                            .tint(ink)
                            .fit_to_exact_size(sz),
                        fmt,
                    );
                    if ui.add(format).on_hover_text("Toggle 24h / 12h (F)").clicked() {
                        self.dispatch(Command::ToggleFormat, ctx, h, mi);
                    }
                    let gear = egui::Button::image_and_text(
                        egui::Image::new(egui::include_image!("../assets/icons/gear-six.svg"))
                            .tint(ink)
                            .fit_to_exact_size(sz),
                        "Settings",
                    );
                    if ui
                        .add(gear)
                        .on_hover_text("Theme, colours, brightness, background (S)")
                        .clicked()
                    {
                        self.dispatch(Command::ToggleSettings, ctx, h, mi);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("VOICEVOX:No.7（アナウンス）").size(12.0).color(dim))
                            .on_hover_text("Japanese voice — attribution required");
                    });
                });
            });
    }

    /// The settings panel (theme, colours, brightness, background images).
    fn settings_window(&mut self, ctx: &egui::Context) {
        let mut open = self.settings_open;
        egui::Window::new("Settings")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.strong("Theme");
                ui.horizontal_wrapped(|ui| {
                    for (name, th) in Theme::PRESETS {
                        if ui.selectable_label(self.theme == th, name).clicked() {
                            self.theme = th;
                        }
                    }
                });
                ui.add_space(4.0);
                egui::Grid::new("colours").spacing([12.0, 6.0]).show(ui, |ui| {
                    ui.label("Accent");
                    ui.color_edit_button_srgba(&mut self.theme.accent);
                    ui.end_row();
                    ui.label("Text");
                    ui.color_edit_button_srgba(&mut self.theme.ink);
                    ui.end_row();
                    ui.label("Background");
                    ui.color_edit_button_srgba(&mut self.theme.bg);
                    ui.end_row();
                });
                ui.add(egui::Slider::new(&mut self.brightness, 0.25..=1.0).text("Brightness"));
                ui.horizontal(|ui| {
                    ui.label("Text size");
                    for (name, v) in [
                        ("Compact", 0.85_f32),
                        ("Standard", 1.0),
                        ("Large", 1.25),
                        ("Huge", 1.5),
                    ] {
                        if ui
                            .selectable_label((self.text_scale - v).abs() < 0.01, name)
                            .clicked()
                        {
                            self.text_scale = v;
                        }
                    }
                });

                ui.separator();
                ui.strong("Background images");
                ui.horizontal(|ui| {
                    ui.label("Folder");
                    ui.text_edit_singleline(&mut self.background.dir);
                    if ui.button("Scan").clicked() {
                        self.background.scan();
                    }
                });
                if !self.background.status.is_empty() {
                    ui.small(self.background.status.clone());
                }
                ui.horizontal_wrapped(|ui| {
                    for m in [BgMode::None, BgMode::Fixed, BgMode::Slideshow, BgMode::Random] {
                        ui.selectable_value(&mut self.background.mode, m, m.label());
                    }
                });
                ui.add(egui::Slider::new(&mut self.background.blur, 0.0..=25.0).text("Blur"));
                ui.add(egui::Slider::new(&mut self.background.scrim, 0.0..=0.9).text("Dim overlay"));
                if matches!(self.background.mode, BgMode::Slideshow | BgMode::Random) {
                    ui.add(
                        egui::Slider::new(&mut self.background.interval_s, 3.0..=120.0).text("Interval (s)"),
                    );
                }
                if self.background.mode == BgMode::Fixed && self.background.images.len() > 1 {
                    ui.horizontal(|ui| {
                        let n = self.background.images.len();
                        if ui.button("◀ prev").clicked() {
                            self.background.idx = (self.background.idx + n - 1) % n;
                        }
                        ui.label(format!("{} / {}", self.background.idx + 1, n));
                        if ui.button("next ▶").clicked() {
                            self.background.idx = (self.background.idx + 1) % n;
                        }
                    });
                }
            });
        self.settings_open = open;
    }
}

impl eframe::App for ClockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
        self.background.tick(ctx.input(|i| i.time));
        apply_theme(ctx, &self.theme);

        let now = Local::now();
        let (y, mo, d) = (now.year(), now.month() as u8, now.day() as u8);
        let (h, mi) = (now.hour() as u8, now.minute() as u8);

        for (key, cmd) in [
            (egui::Key::Space, Command::SpeakTime),
            (egui::Key::R, Command::ToggleRomaji),
            (egui::Key::F, Command::ToggleFormat),
            (egui::Key::S, Command::ToggleSettings),
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

        let theme = self.theme;
        let bg_tex = self.background.texture(ctx);
        let scrim = self.background.scrim;
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme.bg).inner_margin(24.0))
            .show(ctx, |ui| {
                if let Some(tex) = &bg_tex {
                    let rect = ui.max_rect();
                    let uv = cover_uv(tex.size(), rect.size());
                    ui.painter().image(tex.id(), rect, uv, Color32::WHITE);
                    let a = (scrim.clamp(0.0, 1.0) * 255.0) as u8;
                    let sc = Color32::from_rgba_unmultiplied(theme.bg.r(), theme.bg.g(), theme.bg.b(), a);
                    ui.painter().rect_filled(rect, 0.0, sc);
                }
                let s = (ui.available_height() / 720.0).clamp(0.5, 1.2) * self.text_scale;
                ui.vertical_centered(|ui| {
                    ui.add_space(6.0 * s);
                    label(
                        ui,
                        &jptime::weekday_en(y, mo, d).to_uppercase(),
                        62.0 * s,
                        theme.ink,
                    );
                    label(ui, &jptime::part_of_day(h).to_uppercase(), 32.0 * s, theme.dim);
                    ui.add_space(4.0 * s);
                    label(ui, &clock, 150.0 * s, theme.ink);
                    label(ui, &format!("{d} {} {y}", month_en(mo)), 32.0 * s, theme.ink);
                    ui.add_space(10.0 * s);
                    rule(ui, s, theme.accent);
                    ui.add_space(10.0 * s);
                    label(ui, &t.kanji, 54.0 * s, theme.accent);
                    label(ui, &t.kana, 26.0 * s, theme.dim);
                    if self.show_romaji {
                        label(ui, &t.romaji, 24.0 * s, theme.dim);
                    }
                    ui.add_space(4.0 * s);
                    label(ui, &dt.kanji, 32.0 * s, theme.ink);
                    if self.show_romaji {
                        label(ui, &dt.romaji, 20.0 * s, theme.dim);
                    }
                });
            });

        self.settings_window(ctx);

        // brightness: a uniform dimmer over everything (nice at night)
        let a = ((1.0 - self.brightness) * 210.0) as u8;
        if a > 0 {
            let r = ctx.screen_rect();
            ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("dimmer"),
            ))
            .rect_filled(r, 0.0, Color32::from_black_alpha(a));
        }
    }
}

/// UV sub-rect so an image covers `rect` (centre-crop) without distortion.
fn cover_uv(tex: [usize; 2], rect: egui::Vec2) -> egui::Rect {
    let img_a = tex[0] as f32 / tex[1].max(1) as f32;
    let r_a = rect.x / rect.y.max(1.0);
    let (mut ux, mut uy) = (0.0, 0.0);
    if img_a > r_a {
        ux = (1.0 - r_a / img_a) / 2.0;
    } else {
        uy = (1.0 - img_a / r_a) / 2.0;
    }
    egui::Rect::from_min_max(egui::pos2(ux, uy), egui::pos2(1.0 - ux, 1.0 - uy))
}

fn label(ui: &mut egui::Ui, text: &str, size: f32, color: Color32) {
    ui.label(RichText::new(text).size(size).color(color).strong());
}

fn rule(ui: &mut egui::Ui, scale: f32, color: Color32) {
    let w = ui.available_width() * 0.55;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 3.0 * scale), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, color);
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

/// Apply the theme's colours as a flat (no-rounding) egui visual style.
fn apply_theme(ctx: &egui::Context, t: &Theme) {
    let mut v = if t.is_light() {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };
    v.panel_fill = t.bg;
    v.window_fill = t.bg;
    v.extreme_bg_color = t.bg;
    v.override_text_color = Some(t.ink);
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
