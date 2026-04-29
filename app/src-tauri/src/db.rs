use rusqlite::{params, Connection, Result};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct Event {
    pub id: i64,
    pub timestamp: String,
    pub summary: String,
    pub category: String,
    pub app_name: Option<String>,
    pub duration_s: u32,
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(&path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS events (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp   TEXT NOT NULL,
                image_hash  TEXT NOT NULL,
                summary     TEXT NOT NULL,
                category    TEXT NOT NULL,
                app_name    TEXT,
                duration_s  INTEGER DEFAULT 10
            )",
        )?;
        Ok(Db { conn })
    }

    pub fn default_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".day-monitor")
            .join("monitor.db")
    }

    pub fn events_for_date(&self, date: &str) -> Result<Vec<Event>> {
        let pattern = format!("{}%", date);
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, summary, category, app_name, duration_s
             FROM events WHERE timestamp LIKE ?1 ORDER BY timestamp",
        )?;
        let rows = stmt.query_map(params![pattern], |row| {
            Ok(Event {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                summary: row.get(2)?,
                category: row.get(3)?,
                app_name: row.get(4)?,
                duration_s: row.get::<_, i64>(5)? as u32,
            })
        })?;
        rows.collect()
    }

    #[cfg(test)]
    pub(crate) fn from_conn(conn: Connection) -> Self {
        Db { conn }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_in_memory() -> Db {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                image_hash TEXT NOT NULL,
                summary TEXT NOT NULL,
                category TEXT NOT NULL,
                app_name TEXT,
                duration_s INTEGER DEFAULT 10
            )",
        )
        .unwrap();
        Db::from_conn(conn)
    }

    #[test]
    fn empty_db_returns_no_events() {
        let db = open_in_memory();
        let events = db.events_for_date("2026-04-29").unwrap();
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn events_for_date_filters_by_date() {
        let db = open_in_memory();
        db.conn
            .execute(
                "INSERT INTO events (timestamp, image_hash, summary, category, app_name, duration_s)
                 VALUES (?1, 'h1', 'coding work', 'coding', 'VS Code', 60)",
                params!["2026-04-29T09:00:00"],
            )
            .unwrap();
        db.conn
            .execute(
                "INSERT INTO events (timestamp, image_hash, summary, category, app_name, duration_s)
                 VALUES (?1, 'h2', 'old work', 'other', NULL, 30)",
                params!["2026-04-28T09:00:00"],
            )
            .unwrap();

        let events = db.events_for_date("2026-04-29").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].summary, "coding work");
        assert_eq!(events[0].app_name.as_deref(), Some("VS Code"));
    }
}
