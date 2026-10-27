# Icons — Phosphor

Bundled from **Phosphor Icons** (https://github.com/phosphor-icons/core), MIT License,
"regular" weight. Monochrome SVGs, tinted to the theme at runtime.

Rendered two ways:
- **Window / taskbar / Alt-Tab icon** — `clock.svg` rasterised by `resvg` at startup (`src/icons.rs`).
- **Toolbar + settings** — drawn by `egui_extras`' SVG loader (`egui::include_image!`), `.tint(...)`.

Bundled: clock, clock-afternoon, speaker-high, speaker-simple-high, translate, gear-six,
sun, moon-stars, palette, sliders-horizontal, image, images, shuffle, arrows-clockwise,
drop-half, lock-simple, lock-simple-open, eye.
