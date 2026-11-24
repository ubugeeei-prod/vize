//! Component passes share the request's original buffer, cancellation and
//! publication guard. The temporary lowering arena ends with the callback.
use super::{SnapshotRefusal, SourceQuery, SourceQueryResult, SourceSnapshot};
use crate::document::DocumentStore;
use vize_atelier_sfc::{NativeScssSession, NativeSfcCompileOptions};
use vize_l0::Allocator;
use vize_l1::container::{
    Vue,
    vue::{DescriptorOptions, L1Ir},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopedComponentQueryError {
    Host(SnapshotRefusal),
    Language,
}

impl SourceQuery {
    /// Select original blocks for a pass that needs only the container.
    pub async fn run_vue_descriptor<R>(
        self,
        documents: &DocumentStore,
        options: DescriptorOptions,
        compute: impl FnOnce(&SourceSnapshot, &L1Ir<'_>) -> R,
    ) -> Result<SourceQueryResult<R>, ScopedComponentQueryError> {
        self.run_vue(documents, |snapshot, arena| {
            let descriptor = Vue.observe_descriptor(arena, snapshot.source(), options);
            compute(snapshot, &descriptor)
        })
        .await
    }

    /// Run one or more synchronous passes on the same scoped Vue component.
    /// The caller can read original diagnostics even after a lowering refusal.
    /// Owned results are published with the usual current-revision guard.
    pub async fn run_scoped_component<R>(
        self,
        documents: &DocumentStore,
        options: NativeSfcCompileOptions<'_>,
        compute: impl FnOnce(&SourceSnapshot, &NativeScssSession<'_, '_>) -> R,
    ) -> Result<SourceQueryResult<R>, ScopedComponentQueryError> {
        self.run_vue(documents, |snapshot, arena| {
            let session = NativeScssSession::new_scoped(arena, snapshot.source(), options);
            compute(snapshot, &session)
        })
        .await
    }

    async fn run_vue<R>(
        self,
        documents: &DocumentStore,
        compute: impl FnOnce(&SourceSnapshot, &Allocator) -> R,
    ) -> Result<SourceQueryResult<R>, ScopedComponentQueryError> {
        if self.snapshot().language_id() != "vue" {
            return Err(ScopedComponentQueryError::Language);
        }
        self.run(documents, |snapshot| async move {
            compute(&snapshot, &Allocator::default())
        })
        .await
        .map_err(ScopedComponentQueryError::Host)
    }
}
