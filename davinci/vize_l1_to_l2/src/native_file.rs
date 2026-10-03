//! Once-observed SFC assembly from genuine descriptor, script and file owners.
//!
//! This first family is ordinary Vue 3 JS/TS Module syntax and the existing
//! preserve-whitespace native template contract. Other dialects, macros,
//! ignored-empty script policy, controls and emitted products remain unfinished.
//! There is no legacy route, second binder or AST traversal here.

use alloc::vec::Vec;
use vize_l0::{Allocator, Span};
use vize_l1::container::{Vue, vue::DescriptorOptions};

mod observe;
mod query;
mod script;
mod template;
pub use observe::{NativeScriptObservation, NativeSfc, NativeSfcObservation};
pub use query::{NativePositionQueryError, NativeReferenceRef};
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

/// Retain all original owners; only the private assembly check grants admission.
pub fn lower_sfc_native<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DescriptorOptions,
) -> NativeSfcObservation<'a> {
    let descriptor = Vue.observe_descriptor(allocator, source, options);
    let mut issues = Vec::new();
    let view = match descriptor.admitted() {
        Ok(view) => view,
        Err(_) => {
            issues.push(NativeSfcIssue {
                container_index: None,
                span: descriptor.root().map(|root| root.whole_block().span()),
                kind: NativeSfcIssueKind::Descriptor,
            });
            return NativeSfcObservation::new(descriptor, Vec::new(), None, None, issues);
        }
    };
    let mut producer = match crate::vue_file::VueFileProducer::new(allocator, source) {
        Ok(producer) => Some(producer),
        Err(error) => {
            issues.push(NativeSfcIssue {
                container_index: None,
                span: Some(view.root().whole_block().span()),
                kind: NativeSfcIssueKind::FileConstruction(error),
            });
            None
        }
    };
    let mut scripts = Vec::new();
    for selected in [view.ordinary(), view.setup()].into_iter().flatten() {
        scripts.push(script::observe(
            allocator,
            selected,
            producer.as_mut(),
            &mut issues,
        ));
    }
    let mut template = view
        .template()
        .map(|selected| template::prepare(allocator, selected, view.template_lang(), &mut issues));
    if let Some(template) = template.as_mut() {
        template.construct(producer.as_mut(), &mut issues);
    }
    let file = producer.map(crate::vue_file::VueFileProducer::finish);
    if file.as_ref().is_some_and(Result::is_err) {
        issues.push(NativeSfcIssue {
            container_index: None,
            span: Some(view.root().whole_block().span()),
            kind: NativeSfcIssueKind::FileRejected,
        });
    }
    NativeSfcObservation::new(descriptor, scripts, template, file, issues)
}
