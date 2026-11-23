//! The one semantic query surface products read (#6871).
//!
//! A [`Model`] borrows one completed [`FileArtifact`] and answers position,
//! resolution and scope-chain questions over its original rows
//! (`docs/davinci/decisions/2026-11-19-6871-query-model.md`). Answers are
//! borrowed handles or visitors over existing rows: no query parses, indexes
//! or allocates, and a refusal is a typed error, never a name fallback.

use crate::file::FileArtifact;

mod position;
mod template;

pub use crate::file::BindingRef;
/// Vocabulary a question or an answer names: lookup namespaces and refusals.
pub use crate::file::{FileIssue, FileIssueKind, Namespace, TemplateIssue};
pub use position::{PositionQueryError, ReferenceRef, ScopeRef};
pub use template::{
    HandlerLocalRef, TemplateQueryError, TemplateSiteRef, TemplateSiteScope, TemplateSymbolRef,
};

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
}

impl<'f, 'a> Model<'f, 'a> {
    #[must_use]
    pub const fn new(file: &'f FileArtifact<'a>) -> Self {
        Self { file }
    }

    /// The File every answer of this model borrows.
    #[must_use]
    pub const fn file(self) -> &'f FileArtifact<'a> {
        self.file
    }
}

#[cfg(any(test, feature = "position-index"))]
pub use position::index::{IndexedModel, PositionIndex};

#[cfg(feature = "script-declaration-queries")]
pub mod script;

#[cfg(any(test, feature = "observed-queries"))]
pub mod vue;

#[cfg(feature = "expression-queries")]
mod expression;
#[cfg(all(feature = "expression-queries", feature = "position-index"))]
pub use expression::ExpressionPositionIndex;
#[cfg(feature = "expression-queries")]
pub use expression::ExpressionReferenceRef;

#[cfg(feature = "template-operand-queries")]
pub use expression::{ResolvedTemplateOperand, TemplateOperandKind};

#[cfg(feature = "setup-component-imports")]
mod component_import;
#[cfg(feature = "setup-component-imports")]
pub use component_import::{ComponentImportIssue, ImportedComponentSite};

#[cfg(feature = "component-prop-queries")]
pub use component_import::props::{ComponentPropIssue, ImportedComponentProp};

#[cfg(feature = "component-listener-queries")]
pub use component_import::listeners::{ComponentListenerIssue, ImportedComponentListener};
