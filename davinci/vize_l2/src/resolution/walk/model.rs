//! A Model operand supplies semantic ReadWrite use without an assignment AST.

use super::{ReferenceSink, ReferenceSource, Resolver, Usage};
use crate::lang::js::NativeModelInput;
use crate::resolution::{ResolutionError, ResolutionErrorKind};
use oxc_ast::ast::Expression;
use vize_l0::Span;

pub(super) fn original<'a>(
    input: &NativeModelInput<'a>,
    sink: &mut impl ReferenceSink<'a>,
) -> Result<(), ResolutionError> {
    let expression = input.expression().ok_or(ResolutionError {
        span: Span::new(0, 0),
        kind: ResolutionErrorKind::UnsupportedSyntax,
    })?;
    let source = ReferenceSource::Expression(*expression);
    let fail = |kind| ResolutionError {
        span: Span::new(0, source.length()),
        kind,
    };
    if !input.matches_expression(expression) {
        return Err(fail(ResolutionErrorKind::InvalidSpan));
    }
    let Expression::Identifier(identifier) = expression.ast else {
        return Err(fail(ResolutionErrorKind::UnsupportedSyntax));
    };
    let checkpoint = sink.checkpoint();
    let mut resolver = Resolver::new(source, sink);
    let result = resolver.identifier(identifier, Usage::ReadWrite, false);
    if result.is_err() {
        resolver.sink.rollback(checkpoint);
    }
    result
}
