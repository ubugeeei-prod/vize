//! The one semantic query surface products read (#6871).
//!
//! A [`Model`] borrows one completed [`L2Ir`] and answers position,
//! resolution and scope-chain questions over its original rows
//! (`docs/davinci/decisions/2026-11-19-6871-query-model.md`). Answers are
//! borrowed handles or visitors over existing rows: no query parses, indexes
//! or allocates, and a refusal is a typed error, never a name fallback.

use crate::file::L2Ir;

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
/// use vize_l2::{file::L2Ir, query::Model};
/// fn discard(l2ir: L2Ir<'_>, offset: u32) {
///     let binding = Model::new(&l2ir).binding_at_offset(offset).unwrap().unwrap();
///     drop(l2ir);
///     let _ = binding.declaration();
/// }
/// ```
#[derive(Clone, Copy)]
pub struct Model<'f, 'a> {
    l2ir: &'f L2Ir<'a>,
}

impl<'f, 'a> Model<'f, 'a> {
    #[must_use]
    pub const fn new(l2ir: &'f L2Ir<'a>) -> Self {
        Self { l2ir }
    }

    /// The File every answer of this model borrows.
    #[must_use]
    pub const fn l2ir(self) -> &'f L2Ir<'a> {
        self.l2ir
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
