use crate::error::DomainError;
use chrono::NaiveDate;
use rusty_money::{Money, iso};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryId(pub i64);

impl std::fmt::Display for EntryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Expense,
    Income,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category(String);

impl Category {
    pub fn new(name: impl Into<String>) -> Result<Self, DomainError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(DomainError::InvalidData(
                "Category cannot be empty".to_string(),
            ));
        }
        Ok(Self(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Category {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub id: EntryId,
    pub kind: EntryKind,
    pub amount: Money<'static, iso::Currency>,
    pub category: Category,
    pub note: Option<String>,
    pub occurred_on: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEntry {
    pub kind: EntryKind,
    pub amount: Money<'static, iso::Currency>,
    pub category: Category,
    pub note: Option<String>,
    pub occurred_on: NaiveDate,
}

impl NewEntry {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.amount.is_negative() || self.amount.is_zero() {
            return Err(DomainError::InvalidData(
                "Amount must be positive".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct EntryFilter {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub category: Option<Category>,
}
