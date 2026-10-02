//! Own or borrow the original host without copying its document store.
#![expect(
    clippy::disallowed_types,
    reason = "queries retain the same shared host allocation through suspension and final publication"
)]

use std::sync::Arc;

use crate::document::DocumentStore;
use crate::server::ServerState;

/// The variants keep physical host ownership; none grant native File admission.
#[derive(Clone)]
pub(super) enum DocumentHost<'host> {
    Borrowed(&'host DocumentStore),
    SharedStore(Arc<DocumentStore>),
    Server(Arc<ServerState>),
}

impl DocumentHost<'_> {
    pub(super) fn documents(&self) -> &DocumentStore {
        match self {
            Self::Borrowed(documents) => documents,
            Self::SharedStore(documents) => documents,
            Self::Server(state) => &state.documents,
        }
    }
}
