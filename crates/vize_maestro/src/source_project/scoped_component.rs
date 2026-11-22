//! Component passes share the actual request's original buffer, cancellation
//! and publication guard. The temporary lowering arena ends with the callback.
use super::{SnapshotRefusal, SourceQuery, SourceQueryResult, SourceSnapshot};
use crate::document::DocumentStore;
use vize_atelier_sfc::{NativeScssSession, NativeSfcCompileOptions};
use vize_l0::Allocator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopedComponentQueryError {
    Host(SnapshotRefusal),
    Language,
}

impl SourceQuery {
    /// Run one or more synchronous passes on the same scoped Vue component.
    /// The caller can read original diagnostics even after a lowering refusal.
    /// Owned results are published with the usual current-revision guard.
    pub async fn run_scoped_component<R>(
        self,
        documents: &DocumentStore,
        options: NativeSfcCompileOptions<'_>,
        compute: impl FnOnce(&SourceSnapshot, &NativeScssSession<'_, '_>) -> R,
    ) -> Result<SourceQueryResult<R>, ScopedComponentQueryError> {
        if self.snapshot().language_id() != "vue" {
            return Err(ScopedComponentQueryError::Language);
        }
        self.run(documents, |snapshot| async move {
            let arena = Allocator::default();
            let session = NativeScssSession::new_scoped(&arena, snapshot.source(), options);
            compute(&snapshot, &session)
        })
        .await
        .map_err(ScopedComponentQueryError::Host)
    }
}
