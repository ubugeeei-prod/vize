use oxc_ast::ast::{Argument, CallExpression, Expression, ObjectExpression};

pub fn extract_call_expression<'a>(expr: &'a Expression<'a>) -> Option<&'a CallExpression<'a>> {
    match expr {
        Expression::CallExpression(call) => Some(call),
        Expression::TSAsExpression(ts_as) => extract_call_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_call_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            extract_call_expression(&ts_non_null.expression)
        }
        Expression::ParenthesizedExpression(paren) => extract_call_expression(&paren.expression),
        _ => None,
    }
}

pub(in crate::script_parser::extract) struct StringLiteralArgument<'a> {
    pub(in crate::script_parser::extract) value: &'a str,
    pub(in crate::script_parser::extract) literal_start: u32,
    pub(in crate::script_parser::extract) literal_end: u32,
    pub(in crate::script_parser::extract) value_start: u32,
    pub(in crate::script_parser::extract) value_end: u32,
}

pub(in crate::script_parser::extract) fn argument_string_literal<'a>(
    argument: &'a Argument<'a>,
) -> Option<StringLiteralArgument<'a>> {
    match argument {
        Argument::StringLiteral(literal) => Some(StringLiteralArgument {
            value: literal.value.as_str(),
            literal_start: literal.span.start,
            literal_end: literal.span.end,
            value_start: literal.span.start.saturating_add(1),
            value_end: literal.span.end.saturating_sub(1),
        }),
        Argument::ParenthesizedExpression(expr) => expression_string_literal(&expr.expression),
        Argument::TSAsExpression(expr) => expression_string_literal(&expr.expression),
        Argument::TSSatisfiesExpression(expr) => expression_string_literal(&expr.expression),
        Argument::TSNonNullExpression(expr) => expression_string_literal(&expr.expression),
        _ => None,
    }
}

pub(in crate::script_parser::extract) fn argument_identifier<'a>(
    argument: &'a Argument<'a>,
) -> Option<&'a str> {
    match argument {
        Argument::Identifier(identifier) => Some(identifier.name.as_str()),
        Argument::ParenthesizedExpression(expr) => expression_identifier(&expr.expression),
        Argument::TSAsExpression(expr) => expression_identifier(&expr.expression),
        Argument::TSSatisfiesExpression(expr) => expression_identifier(&expr.expression),
        Argument::TSNonNullExpression(expr) => expression_identifier(&expr.expression),
        _ => None,
    }
}

pub(in crate::script_parser::extract) fn argument_object<'a>(
    argument: &'a Argument<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match argument {
        Argument::ObjectExpression(object) => Some(object),
        Argument::ParenthesizedExpression(expr) => expression_object(&expr.expression),
        Argument::TSAsExpression(expr) => expression_object(&expr.expression),
        Argument::TSSatisfiesExpression(expr) => expression_object(&expr.expression),
        Argument::TSNonNullExpression(expr) => expression_object(&expr.expression),
        _ => None,
    }
}

pub(in crate::script_parser::extract::common) fn expression_string_literal<'a>(
    expression: &'a Expression<'a>,
) -> Option<StringLiteralArgument<'a>> {
    match expression {
        Expression::StringLiteral(literal) => Some(StringLiteralArgument {
            value: literal.value.as_str(),
            literal_start: literal.span.start,
            literal_end: literal.span.end,
            value_start: literal.span.start.saturating_add(1),
            value_end: literal.span.end.saturating_sub(1),
        }),
        Expression::ParenthesizedExpression(expr) => expression_string_literal(&expr.expression),
        Expression::TSAsExpression(expr) => expression_string_literal(&expr.expression),
        Expression::TSSatisfiesExpression(expr) => expression_string_literal(&expr.expression),
        Expression::TSNonNullExpression(expr) => expression_string_literal(&expr.expression),
        _ => None,
    }
}

fn expression_identifier<'a>(expression: &'a Expression<'a>) -> Option<&'a str> {
    match expression {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        Expression::ParenthesizedExpression(expr) => expression_identifier(&expr.expression),
        Expression::TSAsExpression(expr) => expression_identifier(&expr.expression),
        Expression::TSSatisfiesExpression(expr) => expression_identifier(&expr.expression),
        Expression::TSNonNullExpression(expr) => expression_identifier(&expr.expression),
        _ => None,
    }
}

fn expression_object<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::ParenthesizedExpression(expr) => expression_object(&expr.expression),
        Expression::TSAsExpression(expr) => expression_object(&expr.expression),
        Expression::TSSatisfiesExpression(expr) => expression_object(&expr.expression),
        Expression::TSNonNullExpression(expr) => expression_object(&expr.expression),
        _ => None,
    }
}
