#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
APP_NAME="Day Monitor.app"
APP_BUNDLE="$ROOT_DIR/src-tauri/target/release/bundle/macos/$APP_NAME"
INSTALL_PATH="/Applications/$APP_NAME"
IDENTITY="${CODE_SIGN_IDENTITY:-Day Monitor Self-Signed}"
EXPECTED_BUNDLE_ID="com.jinkunsun.daymonitor"
LAUNCH_AGENT="$HOME/Library/LaunchAgents/Day Monitor.plist"

if [[ ! -d "$APP_BUNDLE" ]]; then
  echo "missing app bundle: $APP_BUNDLE" >&2
  echo "run 'yarn tauri build' first" >&2
  exit 1
fi

echo "Signing $APP_BUNDLE with identity: $IDENTITY"
codesign \
  --force \
  --deep \
  --sign "$IDENTITY" \
  --identifier "$EXPECTED_BUNDLE_ID" \
  --timestamp=none \
  "$APP_BUNDLE"

echo "Verifying signature"
codesign --verify --deep --strict --verbose=2 "$APP_BUNDLE"

echo "Installing to $INSTALL_PATH"
rm -rf "$INSTALL_PATH"
ditto "$APP_BUNDLE" "$INSTALL_PATH"

echo "Refreshing LaunchAgent if present"
if [[ -f "$LAUNCH_AGENT" ]]; then
  launchctl unload "$LAUNCH_AGENT" >/dev/null 2>&1 || true
  launchctl load "$LAUNCH_AGENT" >/dev/null 2>&1 || true
fi

echo "Installed app details:"
codesign -dvvv "$INSTALL_PATH" 2>&1 | sed -n '1,12p'
