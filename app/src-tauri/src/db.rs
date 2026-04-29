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
            );
            CREATE TABLE IF NOT EXISTS api_calls (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp       TEXT NOT NULL,
                input_tokens    INTEGER NOT NULL,
                output_tokens   INTEGER NOT NULL,
                cost_usd        REAL NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_api_calls_ts ON api_calls(timestamp);",
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

    pub fn events_in_range(&self, from: &str, to: &str) -> Result<Vec<Event>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, summary, category, app_name, duration_s
             FROM events WHERE timestamp >= ?1 AND timestamp < ?2 ORDER BY timestamp",
        )?;
        let rows = stmt.query_map(params![from, to], |row| {
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

    pub fn distinct_categories(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT category FROM events ORDER BY category")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.collect()
    }

    pub fn last_event(&self) -> Result<Option<(i64, String, u32)>> {
        let row = self
            .conn
            .query_row(
                "SELECT id, image_hash, duration_s FROM events ORDER BY id DESC LIMIT 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)? as u32,
                    ))
                },
            )
            .ok();
        Ok(row)
    }

    pub fn insert_event(
        &self,
        timestamp: &str,
        image_hash: &str,
        summary: &str,
        category: &str,
        app_name: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO events (timestamp, image_hash, summary, category, app_name)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![timestamp, image_hash, summary, category, app_name],
        )?;
        Ok(())
    }

    pub fn increment_last_duration(&self, seconds: u32) -> Result<()> {
        self.conn.execute(
            "UPDATE events SET duration_s = duration_s + ?1 WHERE id = (SELECT MAX(id) FROM events)",
            params![seconds as i64],
        )?;
        Ok(())
    }

    pub fn insert_api_call(
        &self,
        timestamp: &str,
        input_tokens: u32,
        output_tokens: u32,
        cost_usd: f64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO api_calls (timestamp, input_tokens, output_tokens, cost_usd)
             VALUES (?1, ?2, ?3, ?4)",
            params![timestamp, input_tokens as i64, output_tokens as i64, cost_usd],
        )?;
        Ok(())
    }

    /// Sum tokens and cost for events with timestamp matching the prefix (e.g. "2026-04-29").
    pub fn api_call_totals_for_prefix(&self, prefix: &str) -> Result<(u32, u32, f64, u32)> {
        let pattern = format!("{}%", prefix);
        let row = self.conn.query_row(
            "SELECT
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(cost_usd), 0.0),
                COUNT(*)
             FROM api_calls WHERE timestamp LIKE ?1",
            params![pattern],
            |row| {
                Ok((
                    row.get::<_, i64>(0)? as u32,
                    row.get::<_, i64>(1)? as u32,
                    row.get::<_, f64>(2)?,
                    row.get::<_, i64>(3)? as u32,
                ))
            },
        )?;
        Ok(row)
    }

    pub fn cleanup_older_than(&self, days: u32) -> Result<usize> {
        if days == 0 {
            return Ok(0); // 0 = keep forever
        }
        let cutoff = chrono::Local::now() - chrono::Duration::days(days as i64);
        let cutoff_str = cutoff.format("%Y-%m-%dT%H:%M:%S").to_string();
        let mut n = self
            .conn
            .execute("DELETE FROM events WHERE timestamp < ?1", params![cutoff_str])?;
        n += self
            .conn
            .execute("DELETE FROM api_calls WHERE timestamp < ?1", params![cutoff_str])?;
        Ok(n)
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
    fn events_in_range_filters() {
        let db = open_in_memory();
        db.conn
            .execute(
                "INSERT INTO events (timestamp, image_hash, summary, category, app_name, duration_s)
                 VALUES (?1, 'h', 's', 'coding', NULL, 60)",
                params!["2026-04-28T09:00:00"],
            )
            .unwrap();
        db.conn
            .execute(
                "INSERT INTO events (timestamp, image_hash, summary, category, app_name, duration_s)
                 VALUES (?1, 'h', 's', 'coding', NULL, 60)",
                params!["2026-04-29T09:00:00"],
            )
            .unwrap();
        let r = db
            .events_in_range("2026-04-29T00:00:00", "2026-04-30T00:00:00")
            .unwrap();
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn distinct_categories_returns_unique() {
        let db = open_in_memory();
        db.conn
            .execute(
                "INSERT INTO events (timestamp, image_hash, summary, category, app_name, duration_s)
                 VALUES (?1, 'h', 's', 'coding', NULL, 60)",
                params!["2026-04-29T09:00:00"],
            )
            .unwrap();
        db.conn
            .execute(
                "INSERT INTO events (timestamp, image_hash, summary, category, app_name, duration_s)
                 VALUES (?1, 'h', 's', 'slack', NULL, 60)",
                params!["2026-04-29T10:00:00"],
            )
            .unwrap();
        db.conn
            .execute(
                "INSERT INTO events (timestamp, image_hash, summary, category, app_name, duration_s)
                 VALUES (?1, 'h', 's', 'coding', NULL, 60)",
                params!["2026-04-29T11:00:00"],
            )
            .unwrap();
        let cats = db.distinct_categories().unwrap();
        assert_eq!(cats, vec!["coding".to_string(), "slack".to_string()]);
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
