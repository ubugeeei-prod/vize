//! True binding positions, separate from retained expression positions.

use oxc_ast::ast::FormalParameter;
use oxc_span::GetSpan;
use vize_l0::{Allocator, Span};

use crate::expr::ExprRef;
use crate::expr::js::{JsCoordinateError, JsCoordinates};

mod aliases;
pub use aliases::{AliasError, NativeForAliases};

#[cfg(test)]
pub(crate) mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsBindingError {
    Coordinates(JsCoordinateError),
    OutsideBinding,
}

/// A binding position supplied by an existing adapter or native formal syntax.
#[derive(Debug, Clone, Copy)]
pub enum BindingRef<'a> {
    /// Existing transitional binding representations retain their semantics.
    Expr(ExprRef<'a>),
    /// A genuine authored formal parameter from the retained parser artifact.
    Js(&'a JsBinding<'a>),
}

impl<'a> BindingRef<'a> {
    #[must_use]
    pub const fn source(self) -> &'a str {
        match self {
            Self::Expr(expr) => expr.source(),
            Self::Js(binding) => binding.source,
        }
    }

    #[must_use]
    pub const fn span(self) -> Span {
        match self {
            Self::Expr(expr) => expr.span(),
            Self::Js(binding) => binding.span,
        }
    }

    /// Only compatibility positions are expressions. No AST cast is offered.
    #[must_use]
    pub const fn as_expression(&self) -> Option<&ExprRef<'a>> {
        match self {
            Self::Expr(expr) => Some(expr),
            Self::Js(_) => None,
        }
    }

    /// Transitional expression rewrites cannot mutate a retained formal root.
    pub fn as_expression_mut(&mut self) -> Option<&mut ExprRef<'a>> {
        match self {
            Self::Expr(expr) => Some(expr),
            Self::Js(_) => None,
        }
    }

    #[must_use]
    pub const fn as_js(self) -> Option<&'a JsBinding<'a>> {
        match self {
            Self::Js(binding) => Some(binding),
            Self::Expr(_) => None,
        }
    }
}

impl<'a> From<ExprRef<'a>> for BindingRef<'a> {
    fn from(value: ExprRef<'a>) -> Self {
        Self::Expr(value)
    }
}

/// The original formal root, exact parameter text and shared alias coordinates.
///
/// Its original patterns, defaults, annotations and descendants remain intact.
/// Comments/diagnostics belong to the normal observation owner outside L2's
/// arena. This carrier performs no parse, AST clone or context-analysis pass.
#[derive(Debug)]
pub struct JsBinding<'a> {
    parameter: &'a FormalParameter<'a>,
    source: &'a str,
    span: Span,
    coordinates: &'a JsCoordinates<'a>,
}

impl<'a> JsBinding<'a> {
    /// Retain only a real formal root whose complete range projects exactly.
    /// The producer supplies an already admitted parameter context.
    pub fn from_retained_in(
        allocator: &'a Allocator,
        parameter: &'a FormalParameter<'a>,
        alias_source: &'a str,
        alias_span: Span,
        coordinates: &'a JsCoordinates<'a>,
    ) -> Result<&'a Self, JsBindingError> {
        if !coordinates.matches(alias_source, alias_span) {
            return Err(JsBindingError::Coordinates(
                JsCoordinateError::InvalidIdentity,
            ));
        }
        let decoded = coordinates
            .decoded_span(parameter.span())
            .ok_or(JsBindingError::OutsideBinding)?;
        let source = alias_source
            .get(decoded.start as usize..decoded.end as usize)
            .filter(|text| !text.is_empty())
            .ok_or(JsBindingError::OutsideBinding)?;
        let span = coordinates
            .authored_span(decoded)
            .ok_or(JsBindingError::OutsideBinding)?;
        Ok(allocator.alloc(Self {
            parameter,
            source,
            span,
            coordinates,
        }))
    }

    #[must_use]
    pub const fn parameter(&self) -> &'a FormalParameter<'a> {
        self.parameter
    }

    #[must_use]
    pub const fn source(&self) -> &'a str {
        self.source
    }

    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    pub fn authored_span(&self, span: oxc_span::Span) -> Option<Span> {
        let decoded = self.coordinates.decoded_span(span)?;
        let authored = self.coordinates.authored_span(decoded)?;
        (authored.start >= self.span.start && authored.end <= self.span.end).then_some(authored)
    }

    pub(crate) fn matches_authored_source(&self, file: &str) -> bool {
        self.coordinates.matches_authored_source(file)
    }

    pub(crate) fn same_alias_block(&self, other: &Self) -> bool {
        core::ptr::eq(self.coordinates, other.coordinates)
    }
}

const _: () = assert!(!core::mem::needs_drop::<BindingRef<'static>>());
const _: () = assert!(!core::mem::needs_drop::<JsBinding<'static>>());
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<BindingRef<'_>>() == 16);
    assert!(core::mem::size_of::<Option<BindingRef<'_>>>() == 16);
    assert!(core::mem::size_of::<JsBinding<'_>>() == 40);
};
