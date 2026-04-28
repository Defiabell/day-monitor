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
        pid = int(PID_PATH.read_text().strip())
        try:
            os.kill(pid, 0)  # check process is alive
            click.echo(f'Monitor already running (PID {pid}). Use `stop` first.')
            return
        except ProcessLookupError:
            PID_PATH.unlink(missing_ok=True)  # stale PID file, clean it up

    api_key = os.environ.get('ANTHROPIC_API_KEY')
    if not api_key:
        click.echo('Error: ANTHROPIC_API_KEY environment variable not set.', err=True)
        sys.exit(1)

    proc = subprocess.Popen(
        [sys.executable, str(Path(__file__).resolve()), '_loop'],
        start_new_session=True,
        env=os.environ.copy(),
    )
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
@click.option('--date', default=None, help='YYYY-MM-DD (default: today)')
def status(date):
    """Print a quick activity summary to the terminal (no file generated)."""
    from collections import defaultdict
    from storage import init_db, get_events_for_date

    date_str = date or datetime.now().strftime('%Y-%m-%d')
    conn = init_db(DB_PATH)
    events = get_events_for_date(conn, date_str)

    if not events:
        click.echo(f'No events found for {date_str}')
        return

    total_s = sum(e['duration_s'] for e in events)
    by_category = defaultdict(int)
    for e in events:
        by_category[e['category']] += e['duration_s']

    def fmt(s):
        h, m = divmod(s // 60, 60)
        return f'{h}h{m:02d}m' if h else f'{m}m'

    click.echo(f'\n今日概况 {date_str}  （共 {fmt(total_s)}）\n')

    click.echo('分类统计：')
    for cat, secs in sorted(by_category.items(), key=lambda x: -x[1]):
        bar = '█' * (secs * 20 // total_s)
        click.echo(f'  {cat:<16} {fmt(secs):>7}  {secs*100//total_s:>3}%  {bar}')

    click.echo('\n最近 10 条活动：')
    for e in events[-10:]:
        t = e['timestamp'][11:16]
        click.echo(f"  {t}  [{e['category']:<13}]  {e['summary']}")


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

    # Write PID file so stop/status commands work regardless of how loop was started
    PID_PATH.parent.mkdir(parents=True, exist_ok=True)
    PID_PATH.write_text(str(os.getpid()))

    try:
        conn = init_db(DB_PATH)
        cleanup_old_events(conn, days=30)
        client = anthropic.Anthropic(api_key=api_key)
        loop = MonitorLoop(conn=conn, client=client, interval=20)
        loop.run()
    finally:
        PID_PATH.unlink(missing_ok=True)


if __name__ == '__main__':
    cli()
