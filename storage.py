import sqlite3
from datetime import datetime, timedelta
from pathlib import Path
from typing import Dict, List, Optional


def init_db(db_path) -> sqlite3.Connection:
    if db_path != ':memory:':
        Path(db_path).parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(db_path))
    conn.execute('''
        CREATE TABLE IF NOT EXISTS events (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp   TEXT NOT NULL,
            image_hash  TEXT NOT NULL,
            summary     TEXT NOT NULL,
            category    TEXT NOT NULL,
            app_name    TEXT,
            duration_s  INTEGER DEFAULT 10
        )
    ''')
    conn.commit()
    return conn


def insert_event(conn: sqlite3.Connection, timestamp: str, image_hash: str,
                 summary: str, category: str, app_name: Optional[str] = None) -> None:
    conn.execute(
        'INSERT INTO events (timestamp, image_hash, summary, category, app_name) VALUES (?,?,?,?,?)',
        (timestamp, image_hash, summary, category, app_name)
    )
    conn.commit()


def get_last_event(conn: sqlite3.Connection) -> Optional[Dict]:
    row = conn.execute(
        'SELECT id, image_hash, duration_s FROM events ORDER BY id DESC LIMIT 1'
    ).fetchone()
    if row:
        return {'id': row[0], 'image_hash': row[1], 'duration_s': row[2]}
    return None


def increment_last_duration(conn: sqlite3.Connection, seconds: int = 10) -> None:
    conn.execute(
        'UPDATE events SET duration_s = duration_s + ? WHERE id = (SELECT MAX(id) FROM events)',
        (seconds,)
    )
    conn.commit()


def get_events_for_date(conn: sqlite3.Connection, date_str: str) -> List[Dict]:
    rows = conn.execute(
        'SELECT timestamp, summary, category, app_name, duration_s '
        'FROM events WHERE timestamp LIKE ? ORDER BY timestamp',
        (f'{date_str}%',)
    ).fetchall()
    return [
        {'timestamp': r[0], 'summary': r[1], 'category': r[2],
         'app_name': r[3], 'duration_s': r[4]}
        for r in rows
    ]


def cleanup_old_events(conn: sqlite3.Connection, days: int = 30) -> int:
    cutoff = (datetime.now() - timedelta(days=days)).strftime('%Y-%m-%dT%H:%M:%S')
    cursor = conn.execute('DELETE FROM events WHERE timestamp < ?', (cutoff,))
    conn.commit()
    return cursor.rowcount
