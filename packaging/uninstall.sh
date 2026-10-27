#!/usr/bin/env bash
# Remove what install.sh placed (user-level only).
set -euo pipefail
data="${XDG_DATA_HOME:-$HOME/.local/share}"
bindir="${XDG_BIN_HOME:-$HOME/.local/bin}"
rm -fv "$bindir/linclock" \
       "$data/applications/lin-apuri-tsuru-clock.desktop" \
       "$data/icons/hicolor/scalable/apps/lin-apuri-tsuru-clock.svg"
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$data/applications" || true
echo "✓ uninstalled"
