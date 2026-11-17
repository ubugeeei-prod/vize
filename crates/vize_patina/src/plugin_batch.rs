//! Plugin record batches shared by every plugin host.
//!
//! One SFC template is lowered to public L2 records; a plugin batch carries
//! the record kinds a plugin visits, the dense parent array, and exactly its
//! demanded fact tables. Reports map back to host-owned authored spans. The
//! native JS host (`@vizejs/native`) and the CLI build the same bytes here.

pub mod batch;
pub mod directives;
pub mod document;
pub mod error;
pub mod facts;
pub mod lint;
pub mod production;
