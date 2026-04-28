#!/usr/bin/env python3
import os
import signal
import subprocess
import sys
from datetime import datetime
from pathlib import Path

import click
from dotenv import load_dotenv

sys.path.insert(0, str(Path(__file__).parent))
load_dotenv(Path(__file__).parent / '.env')

DB_PATH = Path.home() / '.day-monitor' / 'monitor.db'
PID_PATH = Path.home() / '.day-monitor' / 'monitor.pid'
OUTPUT_DIR = Path.home() / 'Documents' / 'day-monitor'


@click.group()
def cli():
    pass


@cli.command()
def start():
    """Start background monitor."""
    if PID_PATH.exists():
        pid = PID_PATH.read_text().strip()
        click.echo(f'Monitor already running (PID {pid}). Use `stop` first.')
        return

    api_key = os.environ.get('ANTHROPIC_API_KEY')
    if not api_key:
        click.echo('Error: ANTHROPIC_API_KEY environment variable not set.', err=True)
        sys.exit(1)

    proc = subprocess.Popen(
        [sys.executable, str(Path(__file__).resolve()), '_loop'],
        start_new_session=True,
        env=os.environ.copy(),
    )
    PID_PATH.parent.mkdir(parents=True, exist_ok=True)
    PID_PATH.write_text(str(proc.pid))
    click.echo(f'Monitor started (PID {proc.pid})')


@cli.command()
def stop():
    """Stop background monitor."""
    if not PID_PATH.exists():
        click.echo('Monitor is not running.')
        return
    pid = int(PID_PATH.read_text().strip())
    try:
        os.kill(pid, signal.SIGTERM)
        click.echo(f'Monitor stopped (PID {pid})')
    except ProcessLookupError:
        click.echo('Monitor was not running (stale PID file removed)')
    finally:
        PID_PATH.unlink(missing_ok=True)


@cli.command()
@click.option('--date', default=None, help='YYYY-MM-DD (default: today)')
def report(date):
    """Generate Markdown report for a given date."""
    import anthropic
    from report import generate_report
    from storage import init_db

    date_str = date or datetime.now().strftime('%Y-%m-%d')
    api_key = os.environ.get('ANTHROPIC_API_KEY')
    if not api_key:
        click.echo('Error: ANTHROPIC_API_KEY environment variable not set.', err=True)
        sys.exit(1)

    client = anthropic.Anthropic(api_key=api_key)
    conn = init_db(DB_PATH)
    output_path = generate_report(conn, date_str, OUTPUT_DIR, client)
    if output_path:
        click.echo(f'Report saved to {output_path}')
    else:
        click.echo(f'No events found for {date_str}')


@cli.command()
def install():
    """Install launchd plist for auto-start on login."""
    from daemon import install_launchd
    install_launchd(sys.executable, str(Path(__file__).resolve()))
    click.echo('Installed. Monitor will start automatically on next login.')
    click.echo('To start now: python monitor.py start')


@cli.command()
def uninstall():
    """Uninstall launchd plist."""
    from daemon import uninstall_launchd
    uninstall_launchd()
    click.echo('Uninstalled launchd service.')


@cli.command('_loop', hidden=True)
def _loop():
    """Internal: run monitor loop (invoked by start and launchd)."""
    import anthropic
    from loop import MonitorLoop
    from storage import cleanup_old_events, init_db

    api_key = os.environ.get('ANTHROPIC_API_KEY')
    if not api_key:
        print('Error: ANTHROPIC_API_KEY not set', file=sys.stderr)
        sys.exit(1)

    conn = init_db(DB_PATH)
    cleanup_old_events(conn, days=30)

    client = anthropic.Anthropic(api_key=api_key)
    loop = MonitorLoop(conn=conn, client=client, interval=10)
    loop.run()


if __name__ == '__main__':
    cli()
