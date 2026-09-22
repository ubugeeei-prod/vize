//! Initializer and pattern-source shapes for the `Bindings` extraction.

use oxc_ast::ast::{Argument, Expression};
use vize_carton::CompactString;

use super::{Init, PatternSource};

pub(super) fn pattern_source(init: Option<&Expression<'_>>, array: bool) -> PatternSource {
    let Some(init) = init else {
        return PatternSource::Absent;
    };
    if let Expression::CallExpression(call) = init
        && let Expression::Identifier(callee) = &call.callee
    {
        let callee = callee.name.as_str();
        let wraps_define_props = callee == "withDefaults"
            && matches!(call.arguments.first(), Some(Argument::CallExpression(inner))
                if matches!(&inner.callee, Expression::Identifier(id) if id.name.as_str() == "defineProps"));
        if !array && (callee == "defineProps" || wraps_define_props) {
            return PatternSource::DefineProps;
        }
    }
    if array && matches!(init_shape(init), Init::Call(callee) if callee == "defineModel") {
        return PatternSource::DefineModel;
    }
    if matches!(
        init,
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
    ) {
        return PatternSource::Function;
    }
    PatternSource::Other
}

fn unwrap_transparent<'a>(mut expression: &'a Expression<'a>) -> &'a Expression<'a> {
    loop {
        expression = match expression {
            Expression::ParenthesizedExpression(inner) => &inner.expression,
            Expression::TSAsExpression(inner) => &inner.expression,
            Expression::TSSatisfiesExpression(inner) => &inner.expression,
            Expression::TSNonNullExpression(inner) => &inner.expression,
            _ => return expression,
        };
    }
}

pub(super) fn init_shape(init: &Expression<'_>) -> Init {
    let init_raw = init;
    let init = unwrap_transparent(init);
    match init {
        Expression::CallExpression(call) => match &call.callee {
            Expression::Identifier(callee) => Init::Call(CompactString::new(callee.name.as_str())),
            _ => Init::Other,
        },
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_) => Init::Literal,
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => Init::Literal,
        Expression::UnaryExpression(unary)
            if unary.operator == oxc_ast::ast::UnaryOperator::UnaryNegation
                && matches!(unary.argument, Expression::NumericLiteral(_)) =>
        {
            Init::Literal
        }
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => {
            Init::Function
        }
        Expression::ObjectExpression(_) | Expression::ArrayExpression(_) => Init::Aggregate,
        Expression::Identifier(id) if matches!(init_raw, Expression::Identifier(_)) => {
            Init::Identifier(CompactString::new(id.name.as_str()))
        }
        _ => Init::Other,
    }
}
