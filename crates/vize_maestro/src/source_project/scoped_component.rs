//! Component passes share the request's original buffer, cancellation and
//! publication guard. The temporary lowering arena ends with the callback.
use super::{SnapshotRefusal, SourceQuery, SourceQueryResult, SourceSnapshot};
use crate::document::DocumentStore;
#[cfg(feature = "experimental-scoped-component-session")]
use vize_atelier_sfc::{NativeScssSession, NativeSfcCompileOptions};
use vize_l0::Allocator;
use vize_l1::container::{
    Vue,
    vue::{DescriptorObservation, DescriptorOptions},
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
        compute: impl FnOnce(&SourceSnapshot, &DescriptorObservation<'_>) -> R,
    ) -> Result<SourceQueryResult<R>, ScopedComponentQueryError> {
        self.run_vue_passes(documents, options, |snapshot, _, descriptor| {
            compute(snapshot, descriptor)
        })
        .await
    }

    /// Run requested passes against one original descriptor and one request
    /// arena. Borrowed syntax and lowered regions expire with the callback.
    /// The result is still subject to cancellation and the revision check.
    ///
    /// ```compile_fail
    /// use vize_l1::container::vue::DescriptorOptions;
    /// use vize_maestro::{document::DocumentStore, source_project::SourceQuery};
    /// async fn retain_descriptor(q: SourceQuery, docs: &DocumentStore, options: DescriptorOptions) {
    ///     let _result = q.run_vue_passes(docs, options, |_, _, owner| owner.container()).await;
    /// }
    /// ```
    pub async fn run_vue_passes<R>(
        self,
        documents: &DocumentStore,
        options: DescriptorOptions,
        compute: impl for<'a> FnOnce(&'a SourceSnapshot, &'a Allocator, &DescriptorObservation<'a>) -> R,
    ) -> Result<SourceQueryResult<R>, ScopedComponentQueryError> {
        self.run_vue(documents, |snapshot, arena| {
            let descriptor = Vue.observe_descriptor(arena, snapshot.source(), options);
            compute(snapshot, arena, &descriptor)
        })
        .await
    }

    /// Run one or more synchronous passes on the same scoped Vue component.
    /// The caller can read original diagnostics even after a lowering refusal.
    /// Owned results are published with the usual current-revision guard.
    #[cfg(feature = "experimental-scoped-component-session")]
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
