//! Explicit host policy for a Patina document root.

use vize_atlas::{
    Compilation, CompilationInputError, PlanningContext, ProviderContext, SourceId, SourceInput,
};

/// Whether Patina should evaluate one registered source.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum PatinaDocumentMode {
    #[default]
    Enabled,
    /// Produce an empty artifact report without planning frontend products.
    Disabled,
}

/// Source-scoped host policy, independent from filename conventions.
pub struct PatinaDocumentModeInput;

impl SourceInput for PatinaDocumentModeInput {
    type Value = PatinaDocumentMode;

    const NAME: &'static str = "patina.document-mode";
}

/// Install an explicit Patina mode for one source.
pub fn install_document_mode(
    compilation: &mut Compilation,
    source: SourceId,
    mode: PatinaDocumentMode,
) -> Result<(), CompilationInputError> {
    compilation
        .set_source_input::<PatinaDocumentModeInput>(source, mode)
        .map(|_| ())
}

pub(super) fn is_disabled_in_plan(context: &PlanningContext<'_>) -> bool {
    context
        .source_input::<PatinaDocumentModeInput>()
        .is_some_and(|mode| *mode == PatinaDocumentMode::Disabled)
}

pub(super) fn is_disabled_in_provider(context: &ProviderContext<'_>) -> bool {
    context
        .source_input::<PatinaDocumentModeInput>()
        .is_some_and(|mode| *mode == PatinaDocumentMode::Disabled)
}
