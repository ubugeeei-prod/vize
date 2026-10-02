//! Shared JS/TS semantic production over actual retained language syntax.

pub(crate) mod file;
pub use file::observer::{CallEvent, DeclaredEvent, FileObserver, StatementEvent};
pub use file::{FileProducer, ProgramInput, ProgramInputError, ProgramScope};
