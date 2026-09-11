#!/bin/sh
# Install the portable build for this user. Does not touch saved games.
set -eu
bundle_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
prefix=${SCRAM_INSTALL_PREFIX:-${MUNCH_INSTALL_PREFIX:-"$HOME/.local"}}
case "$prefix" in
  /*) ;;
  *) printf '%s\n' 'Installation prefix must be absolute.' >&2; exit 1 ;;
esac
if [ ! -f "$bundle_dir/omarchy-scram" ]; then
  printf '%s\n' 'Run this script from the extracted Linux bundle.' >&2
  exit 1
fi
install -d "$prefix/bin" "$prefix/share/applications" "$prefix/share/icons/hicolor/128x128/apps" "$prefix/share/licenses/omarchy-scram"
# Stage then rename so an update also works while a previous binary is running.
install -m755 "$bundle_dir/omarchy-scram" "$prefix/bin/.omarchy-scram-new"
mv -f "$prefix/bin/.omarchy-scram-new" "$prefix/bin/omarchy-scram"
install -m644 "$bundle_dir/omarchy-scram.png" "$prefix/share/icons/hicolor/128x128/apps/io.github.tcballard.omarchy-scram.png"
install -m644 "$bundle_dir/LICENSE" "$prefix/share/licenses/omarchy-scram/LICENSE"
install -m644 "$bundle_dir/THIRD_PARTY_LICENSES.txt" "$prefix/share/licenses/omarchy-scram/THIRD_PARTY_LICENSES.txt"
# Escape characters with special meaning in the Desktop Entry Exec grammar.
escaped_prefix=$(printf '%s' "$prefix" | sed 's/\\/\\\\/g; s/"/\\"/g; s/`/\\`/g; s/\$/\\$/g; s/%/%%/g')
{
  printf '%s\n' '[Desktop Entry]' 'Type=Application' 'Name=Omarchy Scram' 'Comment=An original maze chase for Omarchy Arcade'
  printf 'Exec="%s/bin/omarchy-scram"\n' "$escaped_prefix"
  printf '%s\n' 'Icon=io.github.tcballard.omarchy-scram' 'Terminal=false' 'Categories=Game;ArcadeGame;' 'Keywords=maze;chase;retro;arcade;' 'StartupNotify=true' 'StartupWMClass=io.github.tcballard.omarchy-scram'
} > "$prefix/share/applications/io.github.tcballard.omarchy-scram.desktop"
# Keep old terminal shortcuts working, but show only Scram in the launcher.
ln -sfnT omarchy-scram "$prefix/bin/omarchy-munch"
rm -f "$prefix/share/applications/io.github.tcballard.omarchy-munch.desktop" \
  "$prefix/share/icons/hicolor/128x128/apps/io.github.tcballard.omarchy-munch.png" \
  "$prefix/share/icons/hicolor/scalable/apps/io.github.tcballard.omarchy-munch.svg"
if command -v gtk-update-icon-cache >/dev/null 2>&1; then gtk-update-icon-cache -f -t "$prefix/share/icons/hicolor" >/dev/null 2>&1 || true; fi
if command -v update-desktop-database >/dev/null 2>&1; then update-desktop-database "$prefix/share/applications"; fi
printf 'Installed Omarchy Scram. Open it from your app launcher, or run:\n%s/bin/omarchy-scram\n' "$prefix"
