use crate::error::DomainError;
use crate::models::{Entry, EntryFilter, NewEntry};

pub trait EntryRepository {
    fn add(&mut self, entry: NewEntry) -> Result<Entry, DomainError>;
    fn list(&self, filter: EntryFilter) -> Result<Vec<Entry>, DomainError>;
}
