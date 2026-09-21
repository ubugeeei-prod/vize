//! Document management for the LSP server.
//!
//! This module handles document storage, versioning, and incremental changes.

mod store;
mod text;

pub use store::{Document, DocumentStore};
pub use text::DocumentText;
