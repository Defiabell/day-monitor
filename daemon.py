import subprocess
from pathlib import Path

PLIST_LABEL = 'com.daymonitor'
PLIST_PATH = Path.home() / 'Library' / 'LaunchAgents' / f'{PLIST_LABEL}.plist'
LOG_DIR = Path.home() / '.day-monitor'


def install_launchd(python_path: str, script_path: str) -> None:
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    plist = f"""<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{PLIST_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{python_path}</string>
        <string>{script_path}</string>
        <string>_loop</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
    <key>StandardOutPath</key>
    <string>{LOG_DIR}/stdout.log</string>
    <key>StandardErrorPath</key>
    <string>{LOG_DIR}/stderr.log</string>
</dict>
</plist>"""
    PLIST_PATH.parent.mkdir(parents=True, exist_ok=True)
    PLIST_PATH.write_text(plist)
    subprocess.run(['launchctl', 'load', str(PLIST_PATH)], check=True)


def uninstall_launchd() -> None:
    if PLIST_PATH.exists():
        subprocess.run(['launchctl', 'unload', str(PLIST_PATH)], check=False)
        PLIST_PATH.unlink()


def is_installed() -> bool:
    return PLIST_PATH.exists()
