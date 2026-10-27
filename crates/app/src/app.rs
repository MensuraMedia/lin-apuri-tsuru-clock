//! The egui/eframe clock view: themeable flat UI, 24-hour digits, Japanese
//! reading with a live karaoke underline (VOICEVOX), background images, and an
//! in-app settings page.

use crate::audio;
use crate::command::Command;
use crate::settings::{Background, BgMode, Theme};
use chrono::{Datelike, Local, Timelike};
use eframe::egui;
use egui::{Color32, FontFamily, RichText};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Clock,
    Settings,
}

/// Which readable line a spoken utterance belongs to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Line {
    Time,
    Date,
    Weekday,
    PartOfDay,
}

#[cfg(feature = "audio")]
struct Playing {
    which: Line,
    start: std::time::Instant,
    utt: audio::Utterance,
}

/// Launch the desktop window.
///
/// # Errors
/// Returns any error from `eframe`/winit/wgpu while creating the window.
pub fn run() -> eframe::Result<()> {
    let (iw, ih, irgba) =
        crate::icons::rasterize(include_bytes!("../assets/icons/clock.svg"), 256, [255, 212, 0]);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([820.0, 824.0])
            .with_min_inner_size([460.0, 520.0])
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
    page: Page,
    background: Background,
    #[cfg(feature = "audio")]
    rx: Option<std::sync::mpsc::Receiver<(Line, audio::Utterance)>>,
    #[cfg(feature = "audio")]
    playing: Option<Playing>,
}

impl ClockApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_japanese_font(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        #[cfg(feature = "audio")]
        audio::warm_up();
        Self {
            show_romaji: true,
            fmt24: true,
            theme: Theme::default_dark(),
            brightness: 1.0,
            text_scale: 0.85, // Compact by default
            page: Page::Clock,
            background: Background::default(),
            #[cfg(feature = "audio")]
            rx: None,
            #[cfg(feature = "audio")]
            playing: None,
        }
    }

    fn dispatch(&mut self, cmd: Command, ctx: &egui::Context, h: u8, m: u8) {
        match cmd {
            Command::ToggleRomaji => self.show_romaji = !self.show_romaji,
            Command::ToggleFormat => self.fmt24 = !self.fmt24,
            Command::ToggleSettings => {
                self.page = if self.page == Page::Settings {
                    Page::Clock
                } else {
                    Page::Settings
                };
            }
            Command::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Command::SpeakTime => {
                let k = jptime::time_reading(h, m).kanji;
                self.speak_line(Line::Time, &k);
            }
        }
    }

    /// Synthesise and play a line in Japanese (threaded), recording the timeline
    /// so the UI can underline the sound being pronounced.
    fn speak_line(&mut self, which: Line, text: &str) {
        #[cfg(feature = "audio")]
        {
            let (tx, rx) = std::sync::mpsc::channel();
            self.rx = Some(rx);
            self.playing = None;
            let text = text.to_owned();
            std::thread::spawn(move || {
                if let Some(s) = audio::Speaker::detect()
                    && let Some(u) = s.speak(&text)
                {
                    let _ = tx.send((which, u));
                }
            });
        }
        #[cfg(not(feature = "audio"))]
        let _ = (which, text);
    }

    /// Bottom control bar — icon-only (tooltips carry the labels).
    fn controls(&mut self, ctx: &egui::Context, h: u8, mi: u8) {
        let ink = self.theme.ink;
        let bg = self.theme.bg;
        egui::TopBottomPanel::bottom("controls")
            .frame(egui::Frame::new().fill(bg).inner_margin(8.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let sz = egui::vec2(21.0, 21.0);
                    if icon_button(
                        ui,
                        egui::include_image!("../assets/icons/speaker-high.svg"),
                        ink,
                        sz,
                    )
                    .on_hover_text("Speak the time in Japanese (Space)")
                    .clicked()
                    {
                        self.dispatch(Command::SpeakTime, ctx, h, mi);
                    }
                    if icon_button(ui, egui::include_image!("../assets/icons/translate.svg"), ink, sz)
                        .on_hover_text("Show/hide romaji & kana (R)")
                        .clicked()
                    {
                        self.dispatch(Command::ToggleRomaji, ctx, h, mi);
                    }
                    if icon_button(
                        ui,
                        egui::include_image!("../assets/icons/clock-afternoon.svg"),
                        ink,
                        sz,
                    )
                    .on_hover_text("Toggle 24h / 12h (F)")
                    .clicked()
                    {
                        self.dispatch(Command::ToggleFormat, ctx, h, mi);
                    }
                    if icon_button(ui, egui::include_image!("../assets/icons/gear-six.svg"), ink, sz)
                        .on_hover_text("Settings (S)")
                        .clicked()
                    {
                        self.dispatch(Command::ToggleSettings, ctx, h, mi);
                    }
                });
            });
    }

    /// The clock face, with a speaker on each Japanese line and a karaoke underline.
    fn clock_panel(&mut self, ctx: &egui::Context) {
        let now = Local::now();
        let (y, mo, d) = (now.year(), now.month() as u8, now.day() as u8);
        let (h, mi) = (now.hour() as u8, now.minute() as u8);
        let t = jptime::time_reading(h, mi);
        let dt = jptime::date_reading(y, mo, d);
        let clock = if self.fmt24 {
            format!("{h:02}:{mi:02}")
        } else {
            let h12 = if h % 12 == 0 { 12 } else { h % 12 };
            format!("{h12}:{mi:02} {}", if h < 12 { "AM" } else { "PM" })
        };
        let theme = self.theme;
        let bg_tex = self.background.texture(ctx);
        let scrim = self.background.scrim;

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme.bg).inner_margin(24.0))
            .show(ctx, |ui| {
                if let Some(tex) = &bg_tex {
                    let rect = ui.max_rect();
                    ui.painter()
                        .image(tex.id(), rect, cover_uv(tex.size(), rect.size()), Color32::WHITE);
                    let a = (scrim.clamp(0.0, 1.0) * 255.0) as u8;
                    let sc = Color32::from_rgba_unmultiplied(theme.bg.r(), theme.bg.g(), theme.bg.b(), a);
                    ui.painter().rect_filled(rect, 0.0, sc);
                }
                let s = (ui.available_height() / 760.0).clamp(0.5, 1.2) * self.text_scale;
                let wk_en = jptime::weekday_en(y, mo, d).to_uppercase();
                let part_en = jptime::part_of_day(h).to_uppercase();
                let date_en = format!("{d} {} {y}", month_en(mo));
                let jp_wk = jp_weekday(y, mo, d);
                let jp_part = jp_part_of_day(h);
                ui.vertical_centered(|ui| {
                    ui.add_space(6.0 * s);
                    self.speakable(ui, &wk_en, 58.0 * s, theme.ink, Line::Weekday, &jp_wk);
                    self.speakable(ui, &part_en, 30.0 * s, theme.dim, Line::PartOfDay, jp_part);
                    ui.add_space(4.0 * s);
                    self.speakable(ui, &clock, 128.0 * s, theme.ink, Line::Time, &t.kanji);
                    self.speakable(ui, &date_en, 30.0 * s, theme.ink, Line::Date, &dt.kanji);
                    ui.add_space(8.0 * s);
                    rule(ui, s, theme.accent);
                    ui.add_space(8.0 * s);

                    self.speakable(ui, &t.kanji, 46.0 * s, theme.accent, Line::Time, &t.kanji);
                    self.reading_line(ui, Line::Time, &t.kana, 24.0 * s, theme);
                    if self.show_romaji {
                        label(ui, &t.romaji, 22.0 * s, theme.dim);
                    }
                    ui.add_space(6.0 * s);

                    self.speakable(ui, &dt.kanji, 30.0 * s, theme.ink, Line::Date, &dt.kanji);
                    self.reading_line(ui, Line::Date, &dt.kana, 20.0 * s, theme);
                    if self.show_romaji {
                        label(ui, &dt.romaji, 18.0 * s, theme.dim);
                    }
                });
            });
    }

    /// Render a reading line: the karaoke kana if that line is playing, else the
    /// given fallback text.
    fn reading_line(&self, ui: &mut egui::Ui, which: Line, fallback: &str, size: f32, theme: Theme) {
        #[cfg(feature = "audio")]
        if let Some(p) = self.playing.as_ref().filter(|p| p.which == which) {
            karaoke_kana(
                ui,
                &p.utt.moras,
                p.start.elapsed().as_secs_f32(),
                size,
                theme.dim,
                theme.accent,
            );
            return;
        }
        #[cfg(not(feature = "audio"))]
        let _ = which;
        label(ui, fallback, size, theme.dim);
    }

    /// A centred line with a tap-to-hear speaker to its left.
    fn speakable(&mut self, ui: &mut egui::Ui, text: &str, size: f32, color: Color32, line: Line, say: &str) {
        let accent = self.theme.accent;
        ui.horizontal(|ui| {
            let isz = (size * 0.30).clamp(15.0, 28.0);
            let gap = isz * 0.4;
            center_pad(ui, isz + gap + measure(ui.ctx(), text, size));
            if speaker_icon(ui, accent, isz).clicked() {
                self.speak_line(line, say);
            }
            ui.add_space(gap);
            label(ui, text, size, color);
        });
    }

    /// The settings page (fills the window — not a floating window).
    fn settings_page(&mut self, ctx: &egui::Context) {
        let theme = self.theme;
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme.bg).inner_margin(20.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if icon_button(
                        ui,
                        egui::include_image!("../assets/icons/arrow-left.svg"),
                        theme.ink,
                        egui::vec2(18.0, 18.0),
                    )
                    .on_hover_text("Back to the clock (S / Esc)")
                    .clicked()
                    {
                        self.page = Page::Clock;
                    }
                    label(ui, "Settings", 24.0, theme.ink);
                });
                ui.add_space(6.0);
                egui::ScrollArea::vertical().show(ui, |ui| self.settings_body(ui));
            });
    }

    fn settings_body(&mut self, ui: &mut egui::Ui) {
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
            ui.add(egui::Slider::new(&mut self.background.interval_s, 3.0..=120.0).text("Interval (s)"));
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

        ui.separator();
        ui.small("Voice: VOICEVOX No.7「アナウンス」— click a 🔊 on the clock to hear a line; the yellow underline tracks the sound being spoken.");
    }
}

impl eframe::App for ClockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
        self.background.tick(ctx.input(|i| i.time));
        apply_theme(ctx, &self.theme);

        #[cfg(feature = "audio")]
        {
            if let Some(rx) = &self.rx
                && let Ok((which, u)) = rx.try_recv()
            {
                self.playing = Some(Playing {
                    which,
                    start: std::time::Instant::now(),
                    utt: u,
                });
                self.rx = None;
            }
            if let Some(p) = &self.playing {
                if p.start.elapsed().as_secs_f32() > p.utt.total + 0.4 {
                    self.playing = None;
                } else {
                    ctx.request_repaint_after(std::time::Duration::from_millis(30));
                }
            }
        }

        let now = Local::now();
        let (h, mi) = (now.hour() as u8, now.minute() as u8);
        for (key, cmd) in [
            (egui::Key::Space, Command::SpeakTime),
            (egui::Key::R, Command::ToggleRomaji),
            (egui::Key::F, Command::ToggleFormat),
            (egui::Key::S, Command::ToggleSettings),
            (egui::Key::Escape, Command::Quit),
        ] {
            if ctx.input(|i| i.key_pressed(key)) {
                if key == egui::Key::Escape && self.page == Page::Settings {
                    self.page = Page::Clock;
                } else {
                    self.dispatch(cmd, ctx, h, mi);
                }
            }
        }

        match self.page {
            Page::Clock => {
                self.controls(ctx, h, mi);
                self.clock_panel(ctx);
            }
            Page::Settings => self.settings_page(ctx),
        }

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

/// An icon-only button tinted to `tint`.
fn icon_button(
    ui: &mut egui::Ui,
    src: egui::ImageSource<'_>,
    tint: Color32,
    sz: egui::Vec2,
) -> egui::Response {
    ui.add(egui::ImageButton::new(
        egui::Image::new(src).tint(tint).fit_to_exact_size(sz),
    ))
}

/// A frameless speaker icon (the per-line "say this" affordance).
fn speaker_icon(ui: &mut egui::Ui, tint: Color32, sz: f32) -> egui::Response {
    ui.add(
        egui::ImageButton::new(
            egui::Image::new(egui::include_image!("../assets/icons/speaker-simple-high.svg"))
                .tint(tint)
                .fit_to_exact_size(egui::vec2(sz, sz)),
        )
        .frame(false),
    )
    .on_hover_text("Hear this line in Japanese")
}

/// Left padding so a `width`-wide row ends up roughly centred in the column.
fn center_pad(ui: &mut egui::Ui, width: f32) {
    let pad = (ui.available_width() - width).max(0.0) / 2.0;
    ui.add_space(pad);
}

/// Kana with a yellow underline under the mora currently being pronounced.
#[cfg(feature = "audio")]
fn karaoke_kana(
    ui: &mut egui::Ui,
    moras: &[audio::Mora],
    elapsed: f32,
    size: f32,
    color: Color32,
    accent: Color32,
) {
    let full: String = moras.iter().map(|m| m.kana.as_str()).collect();
    let resp = ui.label(RichText::new(&full).size(size).color(color));
    if let Some(i) = moras.iter().position(|m| elapsed >= m.start && elapsed < m.end) {
        let before: String = moras[..i].iter().map(|m| m.kana.as_str()).collect();
        let x0 = resp.rect.left() + measure(ui.ctx(), &before, size);
        let x1 = x0 + measure(ui.ctx(), &moras[i].kana, size);
        let y = resp.rect.bottom() + 2.0;
        ui.painter().line_segment(
            [egui::pos2(x0, y), egui::pos2(x1, y)],
            egui::Stroke::new(3.0, accent),
        );
    }
}

/// Width of `text` at proportional `size` (to place the underline).
fn measure(ctx: &egui::Context, text: &str, size: f32) -> f32 {
    ctx.fonts(|f| {
        f.layout_no_wrap(text.to_owned(), egui::FontId::proportional(size), Color32::WHITE)
            .rect
            .width()
    })
}

fn label(ui: &mut egui::Ui, text: &str, size: f32, color: Color32) {
    ui.label(RichText::new(text).size(size).color(color).strong());
}

fn rule(ui: &mut egui::Ui, scale: f32, color: Color32) {
    let w = ui.available_width() * 0.55;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 3.0 * scale), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.0, color);
}

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

fn jp_weekday(y: i32, m: u8, d: u8) -> String {
    [
        "日曜日",
        "月曜日",
        "火曜日",
        "水曜日",
        "木曜日",
        "金曜日",
        "土曜日",
    ][jptime::weekday_sun0(y, m, d)]
    .to_string()
}

fn jp_part_of_day(h: u8) -> &'static str {
    match jptime::part_of_day(h) {
        "Morning" => "朝",
        "Afternoon" => "午後",
        "Evening" => "夕方",
        _ => "夜",
    }
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
