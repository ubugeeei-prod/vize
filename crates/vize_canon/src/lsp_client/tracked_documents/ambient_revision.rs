//! A lazily held identity for an exact generation of the native input map.
use super::TrackedDocumentTexts;
use std::{
    borrow::Borrow,
    sync::{Arc, OnceLock},
};

#[derive(Debug, Default)]
pub(super) struct TrackedRevision {
    pub(super) number: u64,
    identity: OnceLock<Arc<()>>,
}

impl Borrow<u64> for TrackedRevision {
    fn borrow(&self) -> &u64 {
        &self.number
    }
}

impl TrackedRevision {
    pub(super) fn wrapping_add(&self, amount: u64) -> Self {
        Self {
            number: self.number.wrapping_add(amount),
            identity: OnceLock::new(),
        }
    }
}

impl TrackedDocumentTexts {
    pub(in crate::lsp_client) fn ambient_revision(&self) -> Arc<()> {
        Arc::clone(self.revision.identity.get_or_init(|| Arc::new(())))
    }

    pub(in crate::lsp_client) fn has_ambient_revision(&self, revision: &Arc<()>) -> bool {
        self.revision
            .identity
            .get()
            .is_some_and(|current| Arc::ptr_eq(current, revision))
    }
}
