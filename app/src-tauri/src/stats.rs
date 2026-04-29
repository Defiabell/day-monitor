use crate::db::{Db, Event};
use rusqlite::Result;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct CategoryStat {
    pub category: String,
    pub seconds: u32,
    pub percent: f32,
}

#[derive(Debug, Serialize)]
pub struct TodayStats {
    pub total_seconds: u32,
    pub categories: Vec<CategoryStat>,
    pub current_activity: Option<Event>,
}

pub fn today_stats(db: &Db) -> Result<TodayStats> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    aggregate_for_date(db, &today)
}

pub fn aggregate_for_date(db: &Db, date: &str) -> Result<TodayStats> {
    let events = db.events_for_date(date)?;
    let total_seconds: u32 = events.iter().map(|e| e.duration_s).sum();

    let mut by_category: HashMap<String, u32> = HashMap::new();
    for e in &events {
        *by_category.entry(e.category.clone()).or_insert(0) += e.duration_s;
    }

    let mut categories: Vec<CategoryStat> = by_category
        .into_iter()
        .map(|(category, seconds)| CategoryStat {
            category,
            seconds,
            percent: if total_seconds > 0 {
                seconds as f32 * 100.0 / total_seconds as f32
            } else {
                0.0
            },
        })
        .collect();
    categories.sort_by(|a, b| b.seconds.cmp(&a.seconds));

    let current_activity = events.last().cloned();

    Ok(TodayStats {
        total_seconds,
        categories,
        current_activity,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{params, Connection};

    fn make_db_with_events(rows: &[(&str, &str, &str, u32)]) -> Db {
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
        for (ts, summary, cat, dur) in rows {
            conn.execute(
                "INSERT INTO events (timestamp, image_hash, summary, category, app_name, duration_s)
                 VALUES (?1, 'h', ?2, ?3, NULL, ?4)",
                params![ts, summary, cat, dur],
            )
            .unwrap();
        }
        Db::from_conn(conn)
    }

    #[test]
    fn empty_day_returns_zero_total() {
        let db = make_db_with_events(&[]);
        let s = aggregate_for_date(&db, "2026-04-29").unwrap();
        assert_eq!(s.total_seconds, 0);
        assert!(s.categories.is_empty());
        assert!(s.current_activity.is_none());
    }

    #[test]
    fn categories_summed_and_sorted_by_duration() {
        let db = make_db_with_events(&[
            ("2026-04-29T09:00:00", "code", "coding", 600),
            ("2026-04-29T10:00:00", "slack msg", "slack", 300),
            ("2026-04-29T11:00:00", "more code", "coding", 200),
        ]);
        let s = aggregate_for_date(&db, "2026-04-29").unwrap();
        assert_eq!(s.total_seconds, 1100);
        assert_eq!(s.categories.len(), 2);
        assert_eq!(s.categories[0].category, "coding");
        assert_eq!(s.categories[0].seconds, 800);
        assert_eq!(s.categories[1].category, "slack");
    }

    #[test]
    fn current_activity_is_last_event() {
        let db = make_db_with_events(&[
            ("2026-04-29T09:00:00", "first", "coding", 60),
            ("2026-04-29T10:00:00", "second", "browser", 30),
        ]);
        let s = aggregate_for_date(&db, "2026-04-29").unwrap();
        assert_eq!(s.current_activity.unwrap().summary, "second");
    }
}
