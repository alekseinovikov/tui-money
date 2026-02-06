use crate::error::DomainError;
use crate::models::{Entry, EntryFilter, NewEntry};
use crate::user::User;

pub trait EntryRepository: UserRepository {
    fn add(&mut self, entry: NewEntry) -> Result<Entry, DomainError>;
    fn list(&self, filter: EntryFilter) -> Result<Vec<Entry>, DomainError>;
}

pub trait UserRepository {
    fn create_user(&mut self, username: &str, password: &str) -> Result<User, DomainError>;
    fn verify_user(&self, username: &str, password: &str) -> Result<Option<User>, DomainError>;
    fn list_users(&self) -> Result<Vec<String>, DomainError>;
}
