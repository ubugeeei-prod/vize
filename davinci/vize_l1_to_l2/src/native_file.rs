//! Once-observed SFC assembly from genuine descriptor, script and file owners.
//!
//! This first family is ordinary Vue 3 JS/TS Module syntax and the existing
//! preserve-whitespace native template contract. Other dialects, macros,
//! ignored-empty script policy, controls and emitted products remain unfinished.
//! There is no legacy route, second binder or AST traversal here.

use vize_l0::{Allocator, Span};
use vize_l1::container::{Vue, vue::DescriptorOptions};

mod observe;
mod query;
mod script;
mod selected;
mod selected_scoped;
mod selected_setup;
mod template;
pub use observe::{NativeScriptObservation, NativeSfc, NativeSfcObservation};
pub use query::{NativePositionQueryError, NativeReferenceRef};
pub use selected::{
    NativeSelectedSfc, NativeSelectedSfcIssue, NativeSelectedSfcIssueKind,
    NativeSelectedSfcObservation, lower_selected_sfc_native,
};
pub use selected_scoped::{
    NativeSelectedScopedSfc, NativeSelectedScopedSfcObservation, lower_selected_scoped_sfc_native,
};
pub use selected_setup::{
    NativeSelectedSetupSfc, NativeSelectedSetupSfcObservation, lower_selected_setup_sfc_native,
};
pub use template::{NativeTemplateObservation, NativeTemplateOutcome};

/// Source-owned orchestration refusal; all underlying observations are retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSfcIssueKind {
    Descriptor,
    FileConstruction(vize_l2::artifact::ArtifactError),
    ScriptSource(vize_l1::embed::SourceError),
    ScriptSyntax(Option<vize_l1::embed::syntax::EmbedHole>),
    /// Structural role presence is not upstream's effective script selection.
    EmptyScriptSelection,
    ScriptInput(vize_l2::lang::js::ProgramInputError),
    VueScript(crate::vue_file::VueFileIssue),
    TemplateSource(vize_l1::markup::ComponentSourceError),
    TemplateFactory(crate::vue_file::VueFileIssue),
    TemplateComponentSource,
    /// The original owner is parked; genuine construction custody is unfinished.
    TemplateConstructionPending,
    TemplateConstruction(crate::native::NativeVueConstructionError),
    TemplateUnsupported,
    FileRejected,
    Membership,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeSfcIssue {
    pub container_index: Option<usize>,
    pub span: Option<Span>,
    pub kind: NativeSfcIssueKind,
}

macro_rules! create_producer {
    ($allocator:expr, $source:expr) => {
        crate::vue_file::VueFileProducer::new($allocator, $source)
    };
}

/// Retain all original owners; only the private assembly check grants admission.
pub fn lower_sfc_native<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DescriptorOptions,
) -> NativeSfcObservation<'a> {
    use template::prepare;
    include!("native_file/assembly.rs")
}

/// Build a Vue 3.4+ File with same-name binding operands owned by the authored arguments.
#[cfg(feature = "authored-bind-shorthand")]
pub fn lower_sfc_with_authored_bind_shorthand<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DescriptorOptions,
) -> NativeSfcObservation<'a> {
    use template::prepare_shorthand as prepare;
    include!("native_file/assembly.rs")
}

#[cfg(feature = "template-operand-queries")]
mod operands;

#[cfg(feature = "setup-component-imports")]
mod component_imports;
#[cfg(feature = "setup-component-imports")]
pub use component_imports::{ComponentImportRefusal, SetupComponentImports};

/// Observe static event names and original method-reference expressions during File construction.
#[cfg(feature = "component-listener-queries")]
pub fn lower_sfc_with_listener_references<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DescriptorOptions,
) -> NativeSfcObservation<'a> {
    use template::prepare_listeners as prepare;
    include!("native_file/assembly.rs")
}

/// Construct original script scopes together with the template's authored
/// bindings and named event references for one semantic tool request.
#[cfg(feature = "lexical-arrow-initializers")]
pub fn lower_sfc_with_lexical_initializers<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DescriptorOptions,
) -> NativeSfcObservation<'a> {
    use template::prepare_lexical as prepare;
    macro_rules! create_producer {
        ($allocator:expr, $source:expr) => {
            crate::vue_file::VueFileProducer::for_lexical_initializers($allocator, $source)
        };
    }
    include!("native_file/assembly.rs")
}
