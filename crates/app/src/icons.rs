//! Rasterise bundled SVG icons to RGBA for the window / taskbar / Alt-Tab icon.
//! Toolbar icons are drawn directly by egui_extras' SVG loader; this is only
//! for `eframe`'s `IconData`, which needs raw pixels at startup.

use resvg::{tiny_skia, usvg};

/// Rasterise `svg` to a `px`-tall RGBA buffer, tinted to `rgb`.
/// The Phosphor icons are monochrome, so we keep the rendered alpha and
/// paint the theme colour through it.
pub fn rasterize(svg: &[u8], px: u32, rgb: [u8; 3]) -> (u32, u32, Vec<u8>) {
    let tree = usvg::Tree::from_data(svg, &usvg::Options::default()).expect("bundled icon is valid SVG");
    let size = tree.size();
    let scale = px as f32 / size.width().max(size.height());
    let w = (size.width() * scale).ceil().max(1.0) as u32;
    let h = (size.height() * scale).ceil().max(1.0) as u32;
    let mut pm = tiny_skia::Pixmap::new(w, h).expect("pixmap alloc");
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pm.as_mut(),
    );
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for p in pm.data().chunks_exact(4) {
        rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], p[3]]);
    }
    (w, h, rgba)
}
