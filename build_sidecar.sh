#!/usr/bin/env bash
# Build the daymonitor-loop binary for embedding into the Tauri .app
set -euo pipefail

cd "$(dirname "$0")"

ARCH="$(uname -m)"
case "$ARCH" in
  arm64) TARGET_TRIPLE=aarch64-apple-darwin ;;
  x86_64) TARGET_TRIPLE=x86_64-apple-darwin ;;
  *) echo "unsupported arch: $ARCH"; exit 1 ;;
esac

OUTPUT_DIR=app/src-tauri/binaries
mkdir -p "$OUTPUT_DIR"

pyinstaller --onefile \
  --name daymonitor-loop \
  --noconfirm \
  --clean \
  loop_entry.py

mv "dist/daymonitor-loop" "$OUTPUT_DIR/daymonitor-loop-$TARGET_TRIPLE"
rm -rf build dist daymonitor-loop.spec

echo "Built $OUTPUT_DIR/daymonitor-loop-$TARGET_TRIPLE"
echo "Size: $(du -h $OUTPUT_DIR/daymonitor-loop-$TARGET_TRIPLE | cut -f1)"
