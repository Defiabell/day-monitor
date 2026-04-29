#!/usr/bin/env python3
"""Pyinstaller entry point for the daymonitor-loop sidecar binary.

Bypasses the click CLI in monitor.py and runs MonitorLoop directly so the
Tauri app can spawn this as a subprocess.
"""
import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from dotenv import load_dotenv

# Load .env from the script's directory (development) and from ~/.day-monitor/.env (production)
load_dotenv(Path(__file__).parent / '.env')
load_dotenv(Path.home() / '.day-monitor' / '.env')


def main() -> None:
    import anthropic
    from loop import MonitorLoop
    from storage import cleanup_old_events, init_db

    api_key = os.environ.get('ANTHROPIC_API_KEY')
    if not api_key:
        print('Error: ANTHROPIC_API_KEY not set', file=sys.stderr)
        sys.exit(1)

    db_path = Path.home() / '.day-monitor' / 'monitor.db'
    pid_path = Path.home() / '.day-monitor' / 'monitor.pid'

    db_path.parent.mkdir(parents=True, exist_ok=True)
    pid_path.write_text(str(os.getpid()))

    try:
        conn = init_db(db_path)
        cleanup_old_events(conn, days=30)
        client = anthropic.Anthropic(api_key=api_key)
        loop = MonitorLoop(conn=conn, client=client, interval=20)
        loop.run()
    finally:
        pid_path.unlink(missing_ok=True)


if __name__ == '__main__':
    main()
