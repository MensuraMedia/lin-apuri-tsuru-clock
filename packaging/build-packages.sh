#!/usr/bin/env bash
# Assemble distributable packages from a release build.
#   AppImage  -> needs `appimagetool` on PATH
#   .deb      -> needs `dpkg-deb`
# Anything missing is skipped with a note; nothing here needs root.
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
ver="$(grep -m1 '^version' "$here/Cargo.toml" | cut -d'"' -f2 || echo 0.0.0)"
out="$here/dist"; mkdir -p "$out"
( cd "$here" && cargo build --release -p linclock )
bin="$here/target/release/linclock"

# ---- AppImage ----
if command -v appimagetool >/dev/null 2>&1; then
  app="$out/AppDir"; rm -rf "$app"
  mkdir -p "$app/usr/bin" "$app/usr/share/applications" \
           "$app/usr/share/icons/hicolor/scalable/apps"
  install -m0755 "$bin" "$app/usr/bin/linclock"
  install -m0644 "$here/packaging/lin-apuri-tsuru-clock.desktop" "$app/lin-apuri-tsuru-clock.desktop"
  cp "$here/packaging/lin-apuri-tsuru-clock.desktop" "$app/usr/share/applications/"
  cp "$here/packaging/app-icon.svg" "$app/usr/share/icons/hicolor/scalable/apps/lin-apuri-tsuru-clock.svg"
  cp "$here/packaging/app-icon.svg" "$app/lin-apuri-tsuru-clock.svg"
  printf '#!/bin/sh\nexec "$APPDIR/usr/bin/linclock" "$@"\n' > "$app/AppRun"; chmod +x "$app/AppRun"
  appimagetool "$app" "$out/lin-apuri-tsuru-clock-$ver-x86_64.AppImage"
  echo "✓ AppImage -> $out"
else echo "– skip AppImage (install appimagetool)"; fi

# ---- .deb ----
if command -v dpkg-deb >/dev/null 2>&1; then
  pkg="$out/deb"; rm -rf "$pkg"
  mkdir -p "$pkg/DEBIAN" "$pkg/usr/bin" "$pkg/usr/share/applications" \
           "$pkg/usr/share/icons/hicolor/scalable/apps"
  install -m0755 "$bin" "$pkg/usr/bin/linclock"
  cp "$here/packaging/lin-apuri-tsuru-clock.desktop" "$pkg/usr/share/applications/"
  cp "$here/packaging/app-icon.svg" "$pkg/usr/share/icons/hicolor/scalable/apps/lin-apuri-tsuru-clock.svg"
  cat > "$pkg/DEBIAN/control" <<CTL
Package: lin-apuri-tsuru-clock
Version: $ver
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Mensura Media
Description: Japanese learning clock (24h time + Japanese readings + audio)
CTL
  dpkg-deb --build --root-owner-group "$pkg" "$out/lin-apuri-tsuru-clock_${ver}_amd64.deb"
  echo "✓ .deb -> $out"
else echo "– skip .deb (install dpkg-deb)"; fi
echo "done → $out"
