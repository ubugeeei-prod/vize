//! Retained host snapshots and cancellable source queries.
//!
//! These adapters use actual open-document revisions. They do not select a
//! production request route or turn a host snapshot into native File admission.

mod query;
mod snapshot;

pub use query::{SourceQuery, SourceQueryResult};
pub use snapshot::{SnapshotKey, SnapshotRefusal, SourceSnapshot, SourceSnapshotCache};

#[cfg(feature = "experimental-source-project")]
mod edit;
#[cfg(feature = "experimental-source-project")]
pub use edit::SourceEditRefusal;

#[cfg(all(test, feature = "experimental-source-project"))]
mod edit_tests;
#[cfg(all(test, feature = "experimental-source-project"))]
mod program_tests;
#[cfg(test)]
mod tests;
