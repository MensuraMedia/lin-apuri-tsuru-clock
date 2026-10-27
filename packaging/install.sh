#!/usr/bin/env bash
# User-level install (no root): builds the release binary and installs a desktop
# launcher + icon, so the app shows up in the program menu, the panel, and Alt-Tab.
set -euo pipefail
here="$(cd "$(dirname "$0")/.." && pwd)"
data="${XDG_DATA_HOME:-$HOME/.local/share}"
bindir="${XDG_BIN_HOME:-$HOME/.local/bin}"
appdir="$data/applications"
icondir="$data/icons/hicolor/scalable/apps"
mkdir -p "$bindir" "$appdir" "$icondir"

echo "› building release binary…"
( cd "$here" && cargo build --release -p linclock )
install -m0755 "$here/target/release/linclock" "$bindir/linclock"
install -m0644 "$here/packaging/app-icon.svg"  "$icondir/lin-apuri-tsuru-clock.svg"
sed "s|^Exec=linclock\$|Exec=$bindir/linclock|" \
    "$here/packaging/lin-apuri-tsuru-clock.desktop" > "$appdir/lin-apuri-tsuru-clock.desktop"
chmod 0644 "$appdir/lin-apuri-tsuru-clock.desktop"

command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$appdir" || true
command -v gtk-update-icon-cache  >/dev/null 2>&1 && gtk-update-icon-cache -f "$data/icons/hicolor" 2>/dev/null || true

echo "✓ installed:"
echo "    binary   $bindir/linclock"
echo "    launcher $appdir/lin-apuri-tsuru-clock.desktop"
echo "    icon     $icondir/lin-apuri-tsuru-clock.svg"
case ":$PATH:" in *":$bindir:"*) ;; *) echo "  note: add $bindir to PATH";; esac
