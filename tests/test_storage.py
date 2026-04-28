import sqlite3
from datetime import datetime, timedelta
import pytest
from storage import init_db, insert_event, get_last_event, increment_last_duration, get_events_for_date, cleanup_old_events


@pytest.fixture
def conn():
    c = init_db(':memory:')
    yield c
    c.close()


def test_init_db_creates_table(conn):
    tables = conn.execute(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='events'"
    ).fetchone()
    assert tables is not None


def test_insert_and_get_last_event(conn):
    insert_event(conn, '2026-04-28T09:00:00', 'aabbccdd', '写代码', 'coding', 'VS Code')
    last = get_last_event(conn)
    assert last['image_hash'] == 'aabbccdd'
    assert last['duration_s'] == 10


def test_increment_last_duration(conn):
    insert_event(conn, '2026-04-28T09:00:00', 'aabbccdd', '写代码', 'coding')
    increment_last_duration(conn, 10)
    last = get_last_event(conn)
    assert last['duration_s'] == 20


def test_get_last_event_empty_db(conn):
    assert get_last_event(conn) is None


def test_get_events_for_date(conn):
    insert_event(conn, '2026-04-28T09:00:00', 'hash1', '写代码', 'coding')
    insert_event(conn, '2026-04-28T10:00:00', 'hash2', '开会', 'meeting')
    insert_event(conn, '2026-04-27T09:00:00', 'hash3', '昨天的', 'other')
    events = get_events_for_date(conn, '2026-04-28')
    assert len(events) == 2
    assert events[0]['summary'] == '写代码'
    assert events[1]['category'] == 'meeting'


def test_cleanup_old_events(conn):
    old_date = (datetime.now() - timedelta(days=31)).strftime('%Y-%m-%dT%H:%M:%S')
    recent_date = datetime.now().strftime('%Y-%m-%dT%H:%M:%S')
    insert_event(conn, old_date, 'oldhash', '旧记录', 'other')
    insert_event(conn, recent_date, 'newhash', '新记录', 'coding')
    deleted = cleanup_old_events(conn, days=30)
    assert deleted == 1
    assert get_events_for_date(conn, old_date[:10]) == []
