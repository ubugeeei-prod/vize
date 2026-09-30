//! Typed expression batches retain producer-supplied signatures and scope.
//! The original expression world remains unchanged. A guest must negotiate
//! `typed-environment@1` before this separate world can be used.

use core::fmt;
use serde::{Deserialize, Serialize};
use vize_l0::String;

use crate::contract::{Capability, GuestError, Span};
use crate::expression::{
    AcceptedAnalysis, Analysis, AnalysisError, Binding, Expression, ExpressionBatch,
    accept_analysis,
};
use crate::handshake::{HandshakeError, Negotiated, negotiate_for};

pub const REQUIRED_FEATURES: &[&str] =
    &["facts-page@1", "projection-page@1", "typed-environment@1"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedBinding {
    pub name: String,
    pub kind: String,
    pub signature: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Demand {
    Value,
    Show,
    Condition,
    Name,
    Handler,
    Statement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedExpression {
    pub id: u32,
    pub source: String,
    pub span: Span,
    pub locals: Vec<TypedBinding>,
    pub expected: Demand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedExpressionBatch {
    pub environment: Vec<TypedBinding>,
    pub expressions: Vec<TypedExpression>,
}

impl TypedExpressionBatch {
    /// The exact environment for one expression, with lexical shadowing.
    #[must_use]
    pub fn scope<'a>(&'a self, expression: &'a TypedExpression) -> Vec<&'a TypedBinding> {
        self.environment
            .iter()
            .filter(|binding| {
                !expression
                    .locals
                    .iter()
                    .any(|local| local.name == binding.name)
            })
            .chain(expression.locals.iter())
            .collect()
    }
}

pub trait TypedExpressionGuest {
    fn get_capability(&mut self) -> Result<Capability, GuestError>;
    fn analyze_typed(&mut self, batch: &TypedExpressionBatch) -> Result<Analysis, GuestError>;
}

impl<G: TypedExpressionGuest + ?Sized> TypedExpressionGuest for Box<G> {
    fn get_capability(&mut self) -> Result<Capability, GuestError> {
        (**self).get_capability()
    }
    fn analyze_typed(&mut self, batch: &TypedExpressionBatch) -> Result<Analysis, GuestError> {
        (**self).analyze_typed(batch)
    }
}

/// Validate shared pages first, then check exact references against the
/// individual expression's scope. A sibling's local is never in scope.
pub fn accept_typed_analysis(
    batch: &TypedExpressionBatch,
    analysis: Analysis,
) -> Result<AcceptedAnalysis, AnalysisError> {
    let untyped = ExpressionBatch {
        environment: batch
            .environment
            .iter()
            .chain(batch.expressions.iter().flat_map(|e| e.locals.iter()))
            .map(|binding| Binding {
                name: binding.name.clone(),
                kind: binding.kind.clone(),
            })
            .collect(),
        expressions: batch
            .expressions
            .iter()
            .map(|e| Expression {
                id: e.id,
                source: e.source.clone(),
                span: e.span,
            })
            .collect(),
    };
    let accepted = accept_analysis(&untyped, analysis)?;
    for expression in &batch.expressions {
        let alpha = &accepted.facts.alpha;
        if alpha.exact.get(&expression.id) == Some(&true) {
            let scope = batch.scope(expression);
            let references = alpha
                .references
                .get(&expression.id)
                .map_or("", |s| s.as_str());
            if let Some(name) = references
                .split(',')
                .filter(|name| !name.is_empty())
                .find(|name| !scope.iter().any(|binding| binding.name == *name))
            {
                return Err(AnalysisError::UnknownBinding {
                    id: expression.id,
                    name: name.into(),
                });
            }
        }
    }
    Ok(accepted)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedExpressionError {
    Guest(GuestError),
    Handshake(HandshakeError),
    Analysis(AnalysisError),
}

impl fmt::Display for TypedExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Guest(error) => error.fmt(f),
            Self::Handshake(error) => write!(f, "handshake refused: {error}"),
            Self::Analysis(error) => write!(f, "analysis refused: {error}"),
        }
    }
}

pub struct TypedExpressionSession<G> {
    guest: G,
    negotiated: Negotiated,
}

impl<G> fmt::Debug for TypedExpressionSession<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypedExpressionSession")
            .field("negotiated", &self.negotiated)
            .finish_non_exhaustive()
    }
}

impl<G: TypedExpressionGuest> TypedExpressionSession<G> {
    pub fn open(mut guest: G) -> Result<Self, TypedExpressionError> {
        let offer = guest
            .get_capability()
            .map_err(TypedExpressionError::Guest)?;
        let negotiated =
            negotiate_for(&offer, REQUIRED_FEATURES).map_err(TypedExpressionError::Handshake)?;
        Ok(Self { guest, negotiated })
    }

    pub fn analyze(
        &mut self,
        batch: &TypedExpressionBatch,
    ) -> Result<AcceptedAnalysis, TypedExpressionError> {
        let analysis = self
            .guest
            .analyze_typed(batch)
            .map_err(TypedExpressionError::Guest)?;
        accept_typed_analysis(batch, analysis).map_err(TypedExpressionError::Analysis)
    }
}
