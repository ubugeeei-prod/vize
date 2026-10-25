//! Post-mutation notification preserves work registered on the new revision.
use super::{SourceQueryProject, abort};
use tower_lsp::lsp_types::Url;

impl SourceQueryProject<'_> {
    #[cfg(feature = "retained-program")]
    pub(in crate::source_project) fn snapshot_is_current(
        &self,
        snapshot: &crate::source_project::SourceSnapshot,
    ) -> bool {
        !self.active.1.load(std::sync::atomic::Ordering::Acquire)
            && !self.host.project_retired()
            && snapshot.check_current(self.host.documents()).is_ok()
    }

    /// Notify only after another owner actually mutates the same live store.
    /// A newly captured query between that mutation and this notification must
    /// survive. Cancel only retired revisions, never all queries for this URI.
    pub fn notify_host_change(&self, uri: &Url) {
        let aborted = {
            let _lifecycle = self.lifecycle.lock();
            self.cache.forget_superseded(self.host.documents(), uri);
            let current = self.host.documents().get(uri).map(|doc| doc.revision());
            let mut removed = Vec::new();
            self.active.0.lock().retain(|query| {
                if &query.uri == uri && Some(query.revision) != current {
                    removed.push(query.cancel.clone());
                    false
                } else {
                    true
                }
            });
            removed
        };
        abort(aborted);
    }
}
