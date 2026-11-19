//! The one semantic query surface products read (#6871).
//!
//! A [`Model`] borrows one completed [`FileArtifact`] and answers position,
//! resolution and scope-chain questions over its original rows, plus Vue
//! binding-kind and contract questions over the facts the caller observed in
//! the same walk (`docs/davinci/decisions/2026-11-19-6871-query-model.md`). Answers are
//! borrowed handles or visitors over existing rows: no query parses, indexes
//! or allocates, and a refusal is a typed error, never a name fallback.

use crate::file::FileArtifact;

mod position;
mod template;
mod vue;

pub use crate::file::BindingRef;
/// Vocabulary a question or an answer names: lookup namespaces and refusals.
pub use crate::file::{FileIssue, FileIssueKind, Namespace, TemplateIssue};
/// Vue vocabulary the binding-kind and contract answers carry.
pub use crate::framework::vue::{
    CallContract, ComponentContract, ComponentOptions, ComponentShape, ContractSource, Emit,
    EmitsContract, MacroCall, MacroKind, ModelContract, ModelName, OptionKey, OptionMember,
    OptionMemberKind, OptionsFacts, Prop, PropsContract, ReactiveSource, SetupBinding,
    SetupBindingKind, SetupFacts, SlotsContract,
};
pub use position::{PositionQueryError, ReferenceRef, ScopeRef};
pub use template::{
    HandlerLocalRef, TemplateQueryError, TemplateSiteRef, TemplateSiteScope, TemplateSymbolRef,
};
pub use vue::{Family, VueQueryError};

/// The query entry over one File. Constructing it does no work.
///
/// Handles borrow the File, not the model, so they outlive a temporary
/// model but never the File:
/// ```compile_fail
/// use vize_l2::{file::FileArtifact, query::Model};
/// fn discard(file: FileArtifact<'_>, offset: u32) {
///     let binding = Model::new(&file).binding_at_offset(offset).unwrap().unwrap();
///     drop(file);
///     let _ = binding.declaration();
/// }
/// ```
#[derive(Clone, Copy)]
pub struct Model<'f, 'a> {
    file: &'f FileArtifact<'a>,
    setup: Option<&'f SetupFacts>,
    options: Option<&'f OptionsFacts>,
}

impl<'f, 'a> Model<'f, 'a> {
    #[must_use]
    pub const fn new(file: &'f FileArtifact<'a>) -> Self {
        Self {
            file,
            setup: None,
            options: None,
        }
    }

    /// The File every answer of this model borrows.
    #[must_use]
    pub const fn file(self) -> &'f FileArtifact<'a> {
        self.file
    }
}
