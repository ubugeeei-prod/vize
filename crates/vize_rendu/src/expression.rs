//! Opaque, owned expressions referenced by render operations.

use crate::RenduProvenance;

/// Producer-provided expression classification useful to backend policy.
///
/// Rendu deliberately does not embed an expression-language AST. A producer
/// may attach code from JavaScript, a template dialect, or another language;
/// semantic products can associate richer analysis by [`RenduExpressionId`](crate::RenduExpressionId).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum RenduExpressionKind {
    Reference,
    Literal,
    Compound,
    Statement,
    #[default]
    Opaque,
}

/// Owned expression material in the expression arena.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RenduExpression {
    pub code: Box<str>,
    pub kind: RenduExpressionKind,
    pub provenance: RenduProvenance,
}

impl RenduExpression {
    pub fn new(code: impl Into<Box<str>>, kind: RenduExpressionKind) -> Self {
        Self {
            code: code.into(),
            kind,
            provenance: RenduProvenance::generated(),
        }
    }

    pub fn with_provenance(mut self, provenance: RenduProvenance) -> Self {
        self.provenance = provenance;
        self
    }
}
