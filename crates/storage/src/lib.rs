use std::path::Path;

use chrono::NaiveDate;
use domain::{Entry, EntryFilter, EntryKind, EntryRepository, NewEntry, RepoError};
use rusqlite::{params, Connection};

const MIGRATIONS: &[(&str, &str)] = &[("001_init.sql", include_str!("../migrations/001_init.sql"))];

const DATE_FORMAT: &str = "%Y-%m-%d";

pub struct SqliteRepository {
    conn: Connection,
}

impl SqliteRepository {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, RepoError> {
        let conn = Connection::open(path.as_ref())
            .map_err(|err| RepoError::Storage(err.to_string()))?;
        let mut repo = Self { conn };
        repo.apply_migrations()?;
        Ok(repo)
    }

    fn apply_migrations(&mut self) -> Result<(), RepoError> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS schema_migrations (
                    version TEXT PRIMARY KEY,
                    applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )
            .map_err(|err| RepoError::Storage(err.to_string()))?;

        let applied = {
            let mut stmt = self
                .conn
                .prepare("SELECT version FROM schema_migrations")
                .map_err(|err| RepoError::Storage(err.to_string()))?;
            stmt.query_map([], |row| row.get::<_, String>(0))
                .map_err(|err| RepoError::Storage(err.to_string()))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|err| RepoError::Storage(err.to_string()))?
        };

        let mut applied_set = std::collections::HashSet::new();
        for version in applied {
            applied_set.insert(version);
        }

        for (version, sql) in MIGRATIONS {
            if applied_set.contains(*version) {
                continue;
            }
            let tx = self
                .conn
                .transaction()
                .map_err(|err| RepoError::Storage(err.to_string()))?;
            tx.execute_batch(sql)
                .map_err(|err| RepoError::Storage(err.to_string()))?;
            tx.execute(
                "INSERT INTO schema_migrations (version) VALUES (?1)",
                [*version],
            )
            .map_err(|err| RepoError::Storage(err.to_string()))?;
            tx.commit()
                .map_err(|err| RepoError::Storage(err.to_string()))?;
        }

        Ok(())
    }

    fn parse_date(value: String) -> Result<NaiveDate, RepoError> {
        NaiveDate::parse_from_str(&value, DATE_FORMAT)
            .map_err(|err| RepoError::InvalidData(err.to_string()))
    }

    fn kind_to_str(kind: EntryKind) -> &'static str {
        match kind {
            EntryKind::Expense => "expense",
            EntryKind::Income => "income",
        }
    }

    fn kind_from_str(value: String) -> Result<EntryKind, RepoError> {
        match value.as_str() {
            "expense" => Ok(EntryKind::Expense),
            "income" => Ok(EntryKind::Income),
            _ => Err(RepoError::InvalidData(format!(
                "unknown entry kind: {value}"
            ))),
        }
    }

}

impl EntryRepository for SqliteRepository {
    fn add(&mut self, entry: NewEntry) -> Result<Entry, RepoError> {
        let kind = Self::kind_to_str(entry.kind);
        let occurred_on = entry.occurred_on.format(DATE_FORMAT).to_string();
        self.conn
            .execute(
                "INSERT INTO entries (kind, amount_cents, category, note, occurred_on)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    kind,
                    entry.amount_cents,
                    entry.category,
                    entry.note,
                    occurred_on
                ],
            )
            .map_err(|err| RepoError::Storage(err.to_string()))?;

        let id = self.conn.last_insert_rowid();
        Ok(Entry {
            id,
            kind: entry.kind,
            amount_cents: entry.amount_cents,
            category: entry.category,
            note: entry.note,
            occurred_on: entry.occurred_on,
        })
    }

    fn list(&self, filter: EntryFilter) -> Result<Vec<Entry>, RepoError> {
        let mut conditions = Vec::new();
        let mut params = Vec::new();

        if let Some(from) = filter.from {
            conditions.push("occurred_on >= ?".to_string());
            params.push(from.format(DATE_FORMAT).to_string());
        }
        if let Some(to) = filter.to {
            conditions.push("occurred_on <= ?".to_string());
            params.push(to.format(DATE_FORMAT).to_string());
        }
        if let Some(category) = filter.category {
            conditions.push("category = ?".to_string());
            params.push(category);
        }

        let mut query = "SELECT id, kind, amount_cents, category, note, occurred_on FROM entries"
            .to_string();
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
        query.push_str(" ORDER BY occurred_on DESC, id DESC");

        let mut stmt = self
            .conn
            .prepare(&query)
            .map_err(|err| RepoError::Storage(err.to_string()))?;

        let mut rows = stmt
            .query(rusqlite::params_from_iter(params))
            .map_err(|err| RepoError::Storage(err.to_string()))?;

        let mut entries = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|err| RepoError::Storage(err.to_string()))?
        {
            let kind: String = row
                .get("kind")
                .map_err(|err| RepoError::Storage(err.to_string()))?;
            let occurred_on: String = row
                .get("occurred_on")
                .map_err(|err| RepoError::Storage(err.to_string()))?;

            entries.push(Entry {
                id: row
                    .get("id")
                    .map_err(|err| RepoError::Storage(err.to_string()))?,
                kind: Self::kind_from_str(kind)?,
                amount_cents: row
                    .get("amount_cents")
                    .map_err(|err| RepoError::Storage(err.to_string()))?,
                category: row
                    .get("category")
                    .map_err(|err| RepoError::Storage(err.to_string()))?,
                note: row
                    .get("note")
                    .map_err(|err| RepoError::Storage(err.to_string()))?,
                occurred_on: Self::parse_date(occurred_on)?,
            });
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db_path(name: &str) -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time moves forward")
            .as_nanos();
        std::env::temp_dir().join(format!("tui-money-{name}-{suffix}.db"))
    }

    #[test]
    fn add_and_list_entries() {
        let path = temp_db_path("add-list");
        let mut repo = SqliteRepository::new(&path).expect("repo created");

        let entry = repo
            .add(NewEntry {
                kind: EntryKind::Expense,
                amount_cents: 1234,
                category: "food".to_string(),
                note: Some("lunch".to_string()),
                occurred_on: NaiveDate::from_ymd_opt(2024, 1, 20).expect("date"),
            })
            .expect("entry added");

        let entries = repo
            .list(EntryFilter::default())
            .expect("entries listed");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], entry);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn list_filters_by_category() {
        let path = temp_db_path("filter-category");
        let mut repo = SqliteRepository::new(&path).expect("repo created");

        repo.add(NewEntry {
            kind: EntryKind::Expense,
            amount_cents: 500,
            category: "food".to_string(),
            note: None,
            occurred_on: NaiveDate::from_ymd_opt(2024, 1, 10).expect("date"),
        })
        .expect("entry added");

        repo.add(NewEntry {
            kind: EntryKind::Income,
            amount_cents: 2500,
            category: "salary".to_string(),
            note: None,
            occurred_on: NaiveDate::from_ymd_opt(2024, 1, 15).expect("date"),
        })
        .expect("entry added");

        let entries = repo
            .list(EntryFilter {
                from: None,
                to: None,
                category: Some("food".to_string()),
            })
            .expect("entries listed");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].category, "food");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn list_filters_by_date_range() {
        let path = temp_db_path("filter-date");
        let mut repo = SqliteRepository::new(&path).expect("repo created");

        repo.add(NewEntry {
            kind: EntryKind::Expense,
            amount_cents: 100,
            category: "food".to_string(),
            note: None,
            occurred_on: NaiveDate::from_ymd_opt(2024, 1, 1).expect("date"),
        })
        .expect("entry added");

        repo.add(NewEntry {
            kind: EntryKind::Expense,
            amount_cents: 200,
            category: "food".to_string(),
            note: None,
            occurred_on: NaiveDate::from_ymd_opt(2024, 1, 10).expect("date"),
        })
        .expect("entry added");

        repo.add(NewEntry {
            kind: EntryKind::Expense,
            amount_cents: 300,
            category: "food".to_string(),
            note: None,
            occurred_on: NaiveDate::from_ymd_opt(2024, 1, 20).expect("date"),
        })
        .expect("entry added");

        let entries = repo
            .list(EntryFilter {
                from: Some(NaiveDate::from_ymd_opt(2024, 1, 5).expect("date")),
                to: Some(NaiveDate::from_ymd_opt(2024, 1, 15).expect("date")),
                category: None,
            })
            .expect("entries listed");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].amount_cents, 200);

        let _ = fs::remove_file(path);
    }
}
