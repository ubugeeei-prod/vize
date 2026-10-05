//! Own or borrow the original host without copying its document store.
#![expect(
    clippy::disallowed_types,
    reason = "queries retain the same shared host allocation through suspension and final publication"
)]

use std::sync::Arc;

use super::{ActiveQueries, AtomicU64, Mutex, SourceQueryProject, SourceSnapshotCache};
use crate::document::DocumentStore;
use crate::server::ServerState;
#[cfg(feature = "experimental-source-navigation")]
mod modules;
mod names;

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

impl<'host> SourceQueryProject<'host> {
    pub(super) fn with_host(host: DocumentHost<'host>) -> Self {
        Self {
            host,
            cache: SourceSnapshotCache::default(),
            active: Arc::new(ActiveQueries::default()),
            next_query: AtomicU64::new(0),
            lifecycle: Mutex::new(()),
        }
    }

    /// Read only actual configuration; publication may hold a document guard.
    /// Never re-enter DocumentStore here or infer server settings for a bare store.
    #[cfg(feature = "experimental-source-navigation")]
    pub(in crate::source_project) fn native_vue_configuration(
        &self,
    ) -> Option<super::super::navigation::profile::VueConfiguration> {
        let DocumentHost::Server(state) = &self.host else {
            return None;
        };
        Some(names::configuration(state.native_names_settings()))
    }
}
