use crate::mapper;
use chrono::NaiveDate;
use domain::{
    Category, DomainError, Entry, EntryFilter, EntryId, EntryRepository, NewEntry, User,
    UserRepository,
};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use rand::rngs::OsRng;

const MIGRATIONS: &[(&str, &str)] = &[
    ("001_init.sql", include_str!("../migrations/001_init.sql")),
    ("002_users.sql", include_str!("../migrations/002_users.sql")),
];
const DATE_FORMAT: &str = "%Y-%m-%d";

pub struct SqliteRepository {
    conn: Connection,
}

impl SqliteRepository {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, DomainError> {
        let conn =
            Connection::open(path.as_ref()).map_err(|err| DomainError::Storage(err.to_string()))?;
        let mut repo = Self { conn };
        repo.apply_migrations()?;
        Ok(repo)
    }

    fn apply_migrations(&mut self) -> Result<(), DomainError> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS schema_migrations (
                    version TEXT PRIMARY KEY,
                    applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )
            .map_err(|err| DomainError::Storage(err.to_string()))?;

        let applied = {
            let mut stmt = self
                .conn
                .prepare("SELECT version FROM schema_migrations")
                .map_err(|err| DomainError::Storage(err.to_string()))?;
            stmt.query_map([], |row| row.get::<_, String>(0))
                .map_err(|err| DomainError::Storage(err.to_string()))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|err| DomainError::Storage(err.to_string()))?
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
                .map_err(|err| DomainError::Storage(err.to_string()))?;
            tx.execute_batch(sql)
                .map_err(|err| DomainError::Storage(err.to_string()))?;
            tx.execute(
                "INSERT INTO schema_migrations (version) VALUES (?1)",
                [*version],
            )
            .map_err(|err| DomainError::Storage(err.to_string()))?;
            tx.commit()
                .map_err(|err| DomainError::Storage(err.to_string()))?;
        }

        Ok(())
    }
}

impl EntryRepository for SqliteRepository {
    fn add(&mut self, entry: NewEntry) -> Result<Entry, DomainError> {
        let kind = mapper::kind_to_str(entry.kind);
        let occurred_on = entry.occurred_on.format(DATE_FORMAT).to_string();
        let amount_cents = mapper::from_money(&entry.amount);
        let category = entry.category.as_str();

        self.conn
            .execute(
                "INSERT INTO entries (kind, amount_cents, category, note, occurred_on)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![kind, amount_cents, category, entry.note, occurred_on],
            )
            .map_err(|err| DomainError::Storage(err.to_string()))?;

        let id = self.conn.last_insert_rowid();
        Ok(Entry {
            id: EntryId(id),
            kind: entry.kind,
            amount: entry.amount,
            category: entry.category,
            note: entry.note,
            occurred_on: entry.occurred_on,
        })
    }

    fn list(&self, filter: EntryFilter) -> Result<Vec<Entry>, DomainError> {
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
            params.push(category.as_str().to_string());
        }

        let mut query =
            "SELECT id, kind, amount_cents, category, note, occurred_on FROM entries".to_string();
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
        query.push_str(" ORDER BY occurred_on DESC, id DESC");

        let mut stmt = self
            .conn
            .prepare(&query)
            .map_err(|err| DomainError::Storage(err.to_string()))?;

        // We need to build params dynamically, but rusqlite expects a trait.
        // We can use split logic or ensure params are strict Strings.
        // params vector is Vec<String>.
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

        let mut rows = stmt
            .query(params_refs.as_slice())
            .map_err(|err| DomainError::Storage(err.to_string()))?;

        let mut entries = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|err| DomainError::Storage(err.to_string()))?
        {
            let id: i64 = row
                .get("id")
                .map_err(|err| DomainError::Storage(err.to_string()))?;
            let kind: String = row
                .get("kind")
                .map_err(|err| DomainError::Storage(err.to_string()))?;
            let amount_cents: i64 = row
                .get("amount_cents")
                .map_err(|err| DomainError::Storage(err.to_string()))?;
            let category_str: String = row
                .get("category")
                .map_err(|err| DomainError::Storage(err.to_string()))?;
            let note: Option<String> = row
                .get("note")
                .map_err(|err| DomainError::Storage(err.to_string()))?;
            let occurred_on_str: String = row
                .get("occurred_on")
                .map_err(|err| DomainError::Storage(err.to_string()))?;

            // Conversions
            let kind = mapper::kind_from_str(kind)?;
            let amount = mapper::to_money(amount_cents);
            let category = Category::new(category_str)?;
            let occurred_on = NaiveDate::parse_from_str(&occurred_on_str, DATE_FORMAT)
                .map_err(|e: chrono::ParseError| DomainError::InvalidData(e.to_string()))?;

            entries.push(Entry {
                id: EntryId(id),
                kind,
                amount,
                category,
                note,
                occurred_on,
            });
        }

        Ok(entries)
    }
}

impl UserRepository for SqliteRepository {
    fn create_user(&mut self, username: &str, password: &str) -> Result<User, DomainError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| DomainError::Storage(format!("Hashing failed: {}", e)))?
            .to_string();

        self.conn
            .execute(
                "INSERT INTO users (username, password_hash) VALUES (?1, ?2)",
                params![username, password_hash],
            )
            .map_err(|err| DomainError::Storage(err.to_string()))?;

        let id = self.conn.last_insert_rowid();

        Ok(User {
            id,
            username: username.to_string(),
        })
    }

    fn verify_user(&self, username: &str, password: &str) -> Result<Option<User>, DomainError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, username, password_hash FROM users WHERE username = ?1")
            .map_err(|err| DomainError::Storage(err.to_string()))?;

        let user_row = stmt
            .query_row([username], |row| {
                let id: i64 = row.get(0)?;
                let username: String = row.get(1)?;
                let password_hash: String = row.get(2)?;
                Ok((id, username, password_hash))
            })
            .optional()
            .map_err(|err| DomainError::Storage(err.to_string()))?;

        if let Some((id, username, password_hash)) = user_row {
            let parsed_hash = PasswordHash::new(&password_hash)
                .map_err(|e| DomainError::Storage(format!("Invalid hash: {}", e)))?;
            
            if Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok()
            {
                return Ok(Some(User {
                    id,
                    username,
                }));
            }
        }

        Ok(None)
    }

    fn list_users(&self) -> Result<Vec<String>, DomainError> {
        let mut stmt = self
            .conn
            .prepare("SELECT username FROM users ORDER BY username")
            .map_err(|err| DomainError::Storage(err.to_string()))?;

        let users = stmt
            .query_map([], |row| row.get(0))
            .map_err(|err| DomainError::Storage(err.to_string()))?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|err| DomainError::Storage(err.to_string()))?;

        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use domain::{Category, EntryFilter, EntryKind, NewEntry};
    use rusty_money::{Money, iso};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db_path(name: &str) -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time moves forward")
            .as_nanos();
        std::env::temp_dir().join(format!("tui-money-{name}-{suffix}.db"))
    }

    fn usd(amount: i64) -> Money<'static, iso::Currency> {
        Money::from_minor(amount, iso::USD)
    }

    #[test]
    fn add_and_list_entries() {
        let path = temp_db_path("add-list");
        let mut repo = SqliteRepository::new(&path).expect("repo created");

        let entry = repo
            .add(NewEntry {
                kind: EntryKind::Expense,
                amount: usd(1234),
                category: Category::new("food").unwrap(),
                note: Some("lunch".to_string()),
                occurred_on: NaiveDate::from_ymd_opt(2024, 1, 20).expect("date"),
            })
            .expect("entry added");

        let entries = repo.list(EntryFilter::default()).expect("entries listed");

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
            amount: usd(500),
            category: Category::new("food").unwrap(),
            note: None,
            occurred_on: NaiveDate::from_ymd_opt(2024, 1, 10).expect("date"),
        })
        .expect("entry added");

        repo.add(NewEntry {
            kind: EntryKind::Income,
            amount: usd(2500),
            category: Category::new("salary").unwrap(),
            note: None,
            occurred_on: NaiveDate::from_ymd_opt(2024, 1, 15).expect("date"),
        })
        .expect("entry added");

        let entries = repo
            .list(EntryFilter {
                from: None,
                to: None,
                category: Some(Category::new("food").unwrap()),
            })
            .expect("entries listed");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].category.as_str(), "food");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn create_and_verify_user() {
        let path = temp_db_path("user-auth");
        let mut repo = SqliteRepository::new(&path).expect("repo created");

        let user = repo
            .create_user("alice", "password123")
            .expect("user created");

        assert_eq!(user.username, "alice");

        let verified = repo
            .verify_user("alice", "password123")
            .expect("verify ok");
        assert_eq!(verified.as_ref().map(|u| u.username.as_str()), Some("alice"));

        let wrong_pass = repo
            .verify_user("alice", "wrong")
            .expect("verify ok (fail)");
        assert!(wrong_pass.is_none());

        let unknown = repo
            .verify_user("bob", "whatever")
            .expect("verify ok (unknown)");
        assert!(unknown.is_none());
        
        // List users
        let users = repo.list_users().expect("list users");
        assert!(users.contains(&"alice".to_string()));

        let _ = fs::remove_file(path);
    }
}
