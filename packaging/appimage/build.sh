#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-only
#
# Builds ZFS Toolbox as an AppImage with linuxdeploy and its GTK plugin.
# Run it on the oldest supported system (Ubuntu 22.04) so that the result
# also starts on newer ones, after:
#   cargo build --release --workspace && scripts/i18n.sh build
#
# Note: the AppImage contains the application only. Administrator actions
# need the helper and its polkit rule installed on the system (from the
# .deb or .rpm package); without them the application stays read-only.

set -eu
cd "$(dirname "$0")/../.."

APP_ID=io.github.gillesmagneville.ZfsToolbox
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
TOOLS=target/appimage-tools
APPDIR=target/AppDir

mkdir -p "$TOOLS"
if [ ! -x "$TOOLS/linuxdeploy-x86_64.AppImage" ]; then
    curl -fsSL -o "$TOOLS/linuxdeploy-x86_64.AppImage" \
        https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
    curl -fsSL -o "$TOOLS/linuxdeploy-plugin-gtk.sh" \
        https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh
    chmod +x "$TOOLS/linuxdeploy-x86_64.AppImage" "$TOOLS/linuxdeploy-plugin-gtk.sh"
fi

rm -rf "$APPDIR"
install -Dm755 target/release/zfs-toolbox "$APPDIR/usr/bin/zfs-toolbox"
for mo in target/data/locale/*/LC_MESSAGES/zfs-toolbox.mo; do
    lang=$(basename "$(dirname "$(dirname "$mo")")")
    install -Dm644 "$mo" "$APPDIR/usr/share/locale/$lang/LC_MESSAGES/zfs-toolbox.mo"
done
install -Dm644 "target/data/$APP_ID.metainfo.xml" "$APPDIR/usr/share/metainfo/$APP_ID.appdata.xml"

export DEPLOY_GTK_VERSION=4
export LINUXDEPLOY_OUTPUT_VERSION="$VERSION"
PATH="$PWD/$TOOLS:$PATH" "$TOOLS/linuxdeploy-x86_64.AppImage" --appimage-extract-and-run \
    --appdir "$APPDIR" \
    --plugin gtk \
    --executable "$APPDIR/usr/bin/zfs-toolbox" \
    --desktop-file "target/data/$APP_ID.desktop" \
    --icon-file "target/data/$APP_ID.svg" \
    --output appimage

mkdir -p dist
mv ZFS_Toolbox-*.AppImage dist/
ls -l dist/
