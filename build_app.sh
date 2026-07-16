#!/usr/bin/env bash
# Full build: sidecar + Tauri app + codesign + install to /Applications
set -euo pipefail

cd "$(dirname "$0")"

BUNDLE_ID="com.jinkunsun.daymonitor"
APP_NAME="Day Monitor"
BUNDLE_DIR="app/src-tauri/target/release/bundle/macos/$APP_NAME.app"

echo "==> Step 1: Build sidecar"
./build_sidecar.sh

echo "==> Step 2: Build Tauri app"
cd app
npm run tauri build
cd ..

echo "==> Step 3: Re-sign with stable identifier ($BUNDLE_ID)"
codesign --force --sign - --identifier "$BUNDLE_ID" "$BUNDLE_DIR/Contents/MacOS/day-monitor"

echo "==> Step 4: Install to /Applications"
osascript -e "tell application \"$APP_NAME\" to quit" 2>/dev/null || true
killall "day-monitor" 2>/dev/null || true
sleep 1
rm -rf "/Applications/$APP_NAME.app"
cp -R "$BUNDLE_DIR" "/Applications/$APP_NAME.app"
xattr -cr "/Applications/$APP_NAME.app"

echo "==> Step 5: Launch"
open "/Applications/$APP_NAME.app"

echo "Done. $APP_NAME installed and running."
