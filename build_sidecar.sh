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

# Use --onedir (NOT --onefile): onefile extracts to /tmp/_MEI<random>/ each launch,
# which causes macOS TCC to re-prompt for screen recording permission every time
# because it sees the Python interpreter as a "new" binary each session.
# --onedir places the runtime in a stable directory; permission persists.
pyinstaller --onedir \
  --name daymonitor-loop \
  --noconfirm \
  --clean \
  loop_entry.py

# Move the entire dist/daymonitor-loop directory to externalBin location.
# The single launchable binary inside is dist/daymonitor-loop/daymonitor-loop;
# Tauri's externalBin expects a single file path with the target triple suffix.
# Solution: bundle the whole _internal directory as a Resource and the launcher
# binary at the externalBin path. We use a helper script that points to the
# real binary inside the .app's Resources.
rm -rf "$OUTPUT_DIR/daymonitor-loop-$TARGET_TRIPLE"
rm -rf "$OUTPUT_DIR/daymonitor-loop.app-runtime"
mv dist/daymonitor-loop "$OUTPUT_DIR/daymonitor-loop.app-runtime"
# Create a launcher binary (no extension on macOS) that execs the real binary
cat > "$OUTPUT_DIR/daymonitor-loop-$TARGET_TRIPLE" <<'LAUNCHER'
#!/bin/sh
DIR="$(cd "$(dirname "$0")" && pwd)"
exec "$DIR/daymonitor-loop.app-runtime/daymonitor-loop" "$@"
LAUNCHER
chmod +x "$OUTPUT_DIR/daymonitor-loop-$TARGET_TRIPLE"
rm -rf build dist daymonitor-loop.spec

echo "Built $OUTPUT_DIR/daymonitor-loop-$TARGET_TRIPLE (launcher)"
echo "Runtime dir: $OUTPUT_DIR/daymonitor-loop.app-runtime"
echo "Total size: $(du -sh $OUTPUT_DIR/ | cut -f1)"
