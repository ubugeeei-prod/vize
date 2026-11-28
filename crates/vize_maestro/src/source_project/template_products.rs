//! Explicit checker/editor products from a genuine original open-buffer File.
use super::{SnapshotRefusal, SourceQuery, SourceQueryResult};
use crate::{
    document::DocumentStore,
    virtual_code::{TemplateOperandProducts, TemplateProductRefusal, VirtualCodeGenerator},
};
use vize_l0::Allocator;
use vize_l1::container::vue::DescriptorOptions;
use vize_l1_to_l2::native_file::{NativeSfcIssue, NativeSfcObservation, lower_sfc_native};

type NativeTemplateProducer =
    for<'a> fn(&'a Allocator, &'a str, DescriptorOptions) -> NativeSfcObservation<'a>;

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
        self.run_native_template_with(documents, options, lower_sfc_native)
            .await
    }

    /// Read original static named listeners and same-name bindings through their
    /// existing native producer. Handler expressions keep the same File/scope
    /// authority as the checker document. Unsupported handler forms still refuse.
    #[cfg(feature = "experimental-component-listeners")]
    pub async fn run_native_listener_template_products(
        self,
        documents: &DocumentStore,
        options: DescriptorOptions,
    ) -> Result<SourceQueryResult<TemplateOperandProducts>, NativeTemplateQueryError> {
        self.run_native_template_with(
            documents,
            options,
            vize_l1_to_l2::native_file::lower_sfc_with_listener_references,
        )
        .await
    }

    /// Read original supported arrow initializer scopes and captures together
    /// with named listener products. This retains the producer's call, parameter
    /// and nested-function refusals and grants no native runtime setup emission.
    #[cfg(feature = "experimental-lexical-arrow-queries")]
    pub async fn run_native_lexical_template_products(
        self,
        documents: &DocumentStore,
        options: DescriptorOptions,
    ) -> Result<SourceQueryResult<TemplateOperandProducts>, NativeTemplateQueryError> {
        self.run_native_template_with(
            documents,
            options,
            vize_l1_to_l2::native_file::lower_sfc_with_lexical_initializers,
        )
        .await
    }

    async fn run_native_template_with(
        self,
        documents: &DocumentStore,
        options: DescriptorOptions,
        produce: NativeTemplateProducer,
    ) -> Result<SourceQueryResult<TemplateOperandProducts>, NativeTemplateQueryError> {
        if self.snapshot().language_id() != "vue" {
            return Err(NativeTemplateQueryError::Language);
        }
        self.run(documents, |snapshot| async move {
            let arena = Allocator::default();
            let original = produce(&arena, snapshot.source(), options);
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
