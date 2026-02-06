mod error;
mod models;
mod repository;

pub use error::DomainError;
// Alias for backward compatibility if needed, or just rename usages
pub use error::DomainError as RepoError;

pub use models::{Category, Entry, EntryFilter, EntryId, EntryKind, NewEntry};
pub use repository::EntryRepository;
