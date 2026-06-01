#!/usr/bin/env python3
"""
Seed ~/.day-monitor/monitor.db with realistic mock activity data (7 days).

Usage:
    python3 scripts/seed_mock_data.py          # append to existing db
    python3 scripts/seed_mock_data.py --reset  # drop and recreate
"""
import sqlite3
import random
import argparse
from datetime import date, datetime, timedelta
from pathlib import Path

DB_PATH = Path.home() / ".day-monitor" / "monitor.db"

WEEKDAY_SCHEDULE = [
    # (hour_start, hour_end, category, app, summary_templates)
    (9,  9,  "email",   "Mail",     ["查看邮件，回复 PR 评论", "处理收件箱，回复客户邮件", "整理邮件，标记待办事项"]),
    (9,  11, "coding",  "Code",     ["在 VS Code 写 Python 代码", "在 VS Code 开发新功能", "Review 代码，合并 PR"]),
    (11, 12, "meeting", "Zoom",     ["Zoom 周会，同步进度", "1:1 会议", "产品评审会议"]),
    (12, 13, "browser", "Chrome",   ["浏览技术文章", "查阅 Stack Overflow", "阅读 Hacker News"]),
    (13, 14, "slack",   "Slack",    ["在 Slack 回复团队消息", "在 Slack 讨论技术方案", "处理 Slack 通知"]),
    (14, 17, "coding",  "Code",     ["在 VS Code 调试 Rust 代码", "实现新功能，写单元测试", "重构代码，优化性能"]),
    (16, 17, "design",  "Figma",    ["在 Figma 设计 UI 原型", "查看设计稿，给出反馈"]),
    (17, 17, "email",   "Mail",     ["发送工作总结邮件", "回复晚间邮件"]),
    (17, 18, "browser", "Chrome",   ["查阅官方文档", "在 Chrome 搜索技术资料"]),
]

CATEGORIES = ["coding", "browser", "slack", "meeting", "email", "design", "writing", "other"]

CATEGORY_COLORS = {
    "coding": "#3b82f6",
    "meeting": "#ef4444",
    "slack": "#a855f7",
    "email": "#f59e0b",
    "browser": "#06b6d4",
    "design": "#ec4899",
    "writing": "#65a30d",
    "other": "#6b7280",
}


def init_db(conn: sqlite3.Connection) -> None:
    conn.execute("""
        CREATE TABLE IF NOT EXISTS events (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp   TEXT NOT NULL,
            image_hash  TEXT NOT NULL,
            summary     TEXT NOT NULL,
            category    TEXT NOT NULL,
            app_name    TEXT,
            duration_s  INTEGER DEFAULT 20
        )
    """)
    conn.execute("""
        CREATE TABLE IF NOT EXISTS cost_log (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp       TEXT NOT NULL,
            input_tokens    INTEGER DEFAULT 0,
            output_tokens   INTEGER DEFAULT 0,
            cost_usd        REAL DEFAULT 0
        )
    """)
    conn.commit()


def fake_hash(i: int) -> str:
    return f"{i:016x}"


def seed_day(conn: sqlite3.Connection, day: date, base_id: int) -> int:
    """Insert ~300 events for a single workday. Returns next base_id."""
    is_weekend = day.weekday() >= 5
    if is_weekend:
        # minimal weekend activity
        slots = [
            (10, 11, "browser", "Chrome", ["浏览新闻，阅读周末文章"]),
            (14, 15, "writing", "Notion", ["整理笔记，写周报"]),
        ]
    else:
        slots = WEEKDAY_SCHEDULE

    event_id = base_id
    for (h_start, h_end, cat, app, summaries) in slots:
        h = h_start
        minute = random.randint(0, 10) if h == 9 else 0
        end_minute = 60 if h_end > h_start else random.randint(10, 50)
        while h < h_end or (h == h_end and minute < end_minute):
            ts = datetime(day.year, day.month, day.day, h, minute, random.randint(0, 59))
            # skip lunch
            if 12 <= ts.hour < 13 and ts.minute < 30 and cat != "browser":
                minute += 20
                if minute >= 60:
                    h += 1
                    minute -= 60
                continue
            duration = random.randint(15, 45)
            summary = random.choice(summaries)
            conn.execute(
                "INSERT INTO events (timestamp, image_hash, summary, category, app_name, duration_s) VALUES (?,?,?,?,?,?)",
                (ts.strftime("%Y-%m-%dT%H:%M:%S"), fake_hash(event_id), summary, cat, app, duration)
            )
            event_id += 1
            minute += duration // 60 * 60 + 20
            if minute >= 60:
                h += minute // 60
                minute = minute % 60
            if h >= 19:
                break

    # cost log entries
    for _ in range(random.randint(20, 40)):
        h = random.randint(9, 17)
        ts = datetime(day.year, day.month, day.day, h, random.randint(0, 59), random.randint(0, 59))
        inp = random.randint(800, 1200)
        out = random.randint(30, 80)
        cost = (inp * 0.25 + out * 1.25) / 1_000_000
        conn.execute(
            "INSERT INTO cost_log (timestamp, input_tokens, output_tokens, cost_usd) VALUES (?,?,?,?)",
            (ts.strftime("%Y-%m-%dT%H:%M:%S"), inp, out, cost)
        )

    conn.commit()
    return event_id


def main() -> None:
    parser = argparse.ArgumentParser(description="Seed Day Monitor with mock data")
    parser.add_argument("--reset", action="store_true", help="Drop and recreate the database")
    parser.add_argument("--days", type=int, default=7, help="Number of days to seed (default: 7)")
    args = parser.parse_args()

    DB_PATH.parent.mkdir(parents=True, exist_ok=True)

    if args.reset and DB_PATH.exists():
        DB_PATH.unlink()
        print(f"Dropped existing database at {DB_PATH}")

    conn = sqlite3.connect(str(DB_PATH))
    init_db(conn)

    today = date.today()
    base_id = 1
    for i in range(args.days - 1, -1, -1):
        day = today - timedelta(days=i)
        base_id = seed_day(conn, day, base_id)
        print(f"  Seeded {day.isoformat()}")

    count = conn.execute("SELECT COUNT(*) FROM events").fetchone()[0]
    conn.close()
    print(f"\nDone. {count} events in {DB_PATH}")


if __name__ == "__main__":
    main()
