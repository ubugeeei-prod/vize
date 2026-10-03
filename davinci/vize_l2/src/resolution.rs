//! Checked identifier uses from an already retained expression AST.
//!
//! This is the expression-level resolution provider, not the file binder.
//! The caller supplies stable binding identities from its enclosing scope.
//! Unsupported syntax stops resolution; an empty successful result therefore
//! means an analyzed expression with no references, never missing facts.

use alloc::vec::Vec;
use vize_l0::Span;

use crate::expr::JsExpr;

pub(crate) mod sink;
mod syntax;
pub use syntax::{SyntaxEdge, SyntaxKind};
pub(crate) mod source;
mod walk;

/// A binding identity supplied by the owning compile unit's binder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingId(u32);

impl BindingId {
    #[must_use]
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// Resolve a semantic identifier name in the enclosing expression scope.
///
/// The same binding must keep its identity across occurrences. Template
/// locals, script declarations and approved globals all need explicit facts;
/// the resolver does not invent context bindings for missing names.
pub trait BindingLookup {
    fn lookup(&self, name: &str) -> Option<BindingId>;
}

/// Whether an identifier is read or is itself an assignment/update target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Usage {
    Read,
    Write,
    ReadWrite,
}

/// One reference, with decoded-source-relative UTF-8 byte coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Occurrence<'a> {
    pub span: Span,
    /// The AST's semantic name, which can differ from an escaped spelling.
    pub name: &'a str,
    pub binding: BindingId,
    pub usage: Usage,
    /// The value of `{ name }`; rewriting it must retain the authored key.
    pub shorthand: bool,
    /// Inside a new callee; helper calls must preserve constructor precedence.
    pub constructor: bool,
}

/// Complete supported-expression facts tied to the exact retained AST/text.
///
/// There is no constructor accepting hand-written occurrence lists.
#[derive(Debug)]
pub struct ResolutionTable<'a> {
    expression: JsExpr<'a>,
    occurrences: Vec<Occurrence<'a>>,
}

impl<'a> ResolutionTable<'a> {
    #[must_use]
    pub fn expression(&self) -> &JsExpr<'a> {
        &self.expression
    }

    #[must_use]
    pub fn occurrences(&self) -> &[Occurrence<'a>] {
        &self.occurrences
    }
}

/// A decoded-relative rejection. No partial resolution table is returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolutionError {
    pub span: Span,
    pub kind: ResolutionErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionErrorKind {
    MissingBinding,
    InvalidSpan,
    UnsupportedSyntax,
    TraversalLimit,
}

/// Resolve the supported JavaScript expression family without parsing again.
///
/// Scoped functions/classes, destructuring targets, TS erasure, JSX and
/// context-sensitive forms require their own complete admission.
/// They return `UnsupportedSyntax`; their identifiers are never silently lost.
pub fn resolve_expression<'a>(
    expression: &JsExpr<'a>,
    bindings: &impl BindingLookup,
) -> Result<ResolutionTable<'a>, ResolutionError> {
    let occurrences = sink::resolve(expression, bindings)?;
    Ok(ResolutionTable {
        expression: *expression,
        occurrences,
    })
}
