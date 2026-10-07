//! Function identity facts emitted by existing declaration/expression walks.

use super::ScriptParseResult;
use oxc_ast::ast::{BindingPattern, Expression, Function, Statement};
use oxc_span::GetSpan;

pub(super) fn note_function(result: &mut ScriptParseResult, function: &Function<'_>) {
    if !result.skip_diagnostics {
        result.setup_context.note_ssr_function(
            (function.span.start, function.span.end),
            function.id.as_ref().map(|id| (id.span.start, id.span.end)),
        );
    }
}

pub(super) fn note_arrow(result: &mut ScriptParseResult, expression: &Expression<'_>) {
    if !result.skip_diagnostics {
        let span = expression.span();
        result
            .setup_context
            .note_ssr_function((span.start, span.end), None);
    }
}

pub(super) fn note_initializer(
    result: &mut ScriptParseResult,
    pattern: &BindingPattern<'_>,
    expression: Option<&Expression<'_>>,
) {
    if result.skip_diagnostics {
        return;
    }
    let (BindingPattern::BindingIdentifier(id), Some(expression)) = (pattern, expression) else {
        return;
    };
    let expression = expression.get_inner_expression();
    if matches!(
        expression,
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
    ) {
        let span = expression.span();
        result
            .setup_context
            .note_ssr_function((span.start, span.end), Some((id.span.start, id.span.end)));
    }
}

pub(super) fn walk_body(
    result: &mut ScriptParseResult,
    statements: &[Statement<'_>],
    source: &str,
    end: u32,
) {
    for statement in statements {
        super::ssr_guards::note_guard(result, statement, Some(end));
        super::walk::walk_statement(result, statement, source);
    }
}
