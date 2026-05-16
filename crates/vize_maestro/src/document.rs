//! Document management for the LSP server.
//!
//! This module handles document storage, versioning, and incremental changes.

mod store;

pub use store::{Document, DocumentStore};
