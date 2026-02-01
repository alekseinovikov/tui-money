use chrono::NaiveDate;
use thiserror::Error;

pub type EntryId = i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Expense,
    Income,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub id: EntryId,
    pub kind: EntryKind,
    pub amount_cents: i64,
    pub category: String,
    pub note: Option<String>,
    pub occurred_on: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEntry {
    pub kind: EntryKind,
    pub amount_cents: i64,
    pub category: String,
    pub note: Option<String>,
    pub occurred_on: NaiveDate,
}

impl NewEntry {
    pub fn validate(&self) -> Result<(), RepoError> {
        if self.amount_cents <= 0 {
            return Err(RepoError::InvalidData(
                "amount must be greater than zero".to_string(),
            ));
        }
        if self.category.trim().is_empty() {
            return Err(RepoError::InvalidData(
                "category must not be empty".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct EntryFilter {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub category: Option<String>,
}

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("storage error: {0}")]
    Storage(String),
    #[error("record not found")]
    NotFound,
    #[error("invalid data: {0}")]
    InvalidData(String),
}

pub trait EntryRepository {
    fn add(&mut self, entry: NewEntry) -> Result<Entry, RepoError>;
    fn list(&self, filter: EntryFilter) -> Result<Vec<Entry>, RepoError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_non_positive_amounts() {
        let entry = NewEntry {
            kind: EntryKind::Expense,
            amount_cents: 0,
            category: "food".to_string(),
            note: None,
            occurred_on: NaiveDate::from_ymd_opt(2024, 2, 1).expect("date"),
        };

        let err = entry.validate().expect_err("expected invalid amount");
        assert!(matches!(err, RepoError::InvalidData(_)));
    }

    #[test]
    fn validate_rejects_empty_category() {
        let entry = NewEntry {
            kind: EntryKind::Income,
            amount_cents: 100,
            category: "   ".to_string(),
            note: None,
            occurred_on: NaiveDate::from_ymd_opt(2024, 2, 1).expect("date"),
        };

        let err = entry.validate().expect_err("expected invalid category");
        assert!(matches!(err, RepoError::InvalidData(_)));
    }

    #[test]
    fn validate_accepts_valid_entry() {
        let entry = NewEntry {
            kind: EntryKind::Expense,
            amount_cents: 2500,
            category: "transport".to_string(),
            note: Some("bus".to_string()),
            occurred_on: NaiveDate::from_ymd_opt(2024, 2, 1).expect("date"),
        };

        entry.validate().expect("entry is valid");
    }
}
