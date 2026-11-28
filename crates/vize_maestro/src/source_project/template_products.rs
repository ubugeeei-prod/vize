//! Explicit checker/editor products from a genuine original open-buffer File.
use super::{SnapshotRefusal, SourceQuery, SourceQueryResult};
use crate::{
    document::DocumentStore,
    virtual_code::{TemplateOperandProducts, TemplateProductRefusal, VirtualCodeGenerator},
};
use vize_l0::Allocator;
use vize_l1::container::vue::DescriptorOptions;
use vize_l1_to_l2::native_file::{NativeSfcIssue, lower_sfc_native};

/// Host freshness and whole-File admission remain separate failure boundaries.
#[derive(Debug, PartialEq, Eq)]
pub enum NativeTemplateQueryError {
    Host(SnapshotRefusal),
    Language,
    File(Vec<NativeSfcIssue>),
    Projection(TemplateProductRefusal),
}

impl SourceQuery {
    /// Build the original Vue File once inside this request's temporary arena,
    /// then share its operands with the checker and editor. The real open-buffer
    /// URI owns the virtual script URI. Unsupported input returns the producer's
    /// typed refusals without a legacy fallback.
    ///
    /// Returned products still require [`SourceQueryResult::publish`], which
    /// rejects cancellation, edits, close/reopen and renamed documents. No
    /// document lock survives native construction. This explicitly requested
    /// product does not select an LSP endpoint or the persistent native backend.
    pub async fn run_native_template_products(
        self,
        documents: &DocumentStore,
        options: DescriptorOptions,
    ) -> Result<SourceQueryResult<TemplateOperandProducts>, NativeTemplateQueryError> {
        if self.snapshot().language_id() != "vue" {
            return Err(NativeTemplateQueryError::Language);
        }
        self.run(documents, |snapshot| async move {
            let arena = Allocator::default();
            let original = lower_sfc_native(&arena, snapshot.source(), options);
            let native = original
                .admitted()
                .ok_or_else(|| NativeTemplateQueryError::File(original.issues().to_vec()))?;
            VirtualCodeGenerator::new()
                .native_template_operand_products(&native, snapshot.uri().as_str())
                .map_err(NativeTemplateQueryError::Projection)
        })
        .await
        .map_err(NativeTemplateQueryError::Host)?
        .transpose()
    }
}

#[cfg(test)]
mod tests;
