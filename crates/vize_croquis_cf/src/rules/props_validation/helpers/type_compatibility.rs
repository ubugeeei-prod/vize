//! Reject static values only when the complete declared type is understood.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement, TSLiteral, TSType, TSTypeName};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::cstr;
use vize_croquis::types::TypeDefinitions;

pub(in super::super) fn prop_type_accepts_actual(
    expected: &str,
    actual: &str,
    definitions: &TypeDefinitions,
) -> bool {
    let source = cstr!("type __Expected = {expected}; type __Actual = {actual};");
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return true;
    }
    let mut statements = parsed.program.body.iter();
    let (
        Some(Statement::TSTypeAliasDeclaration(expected)),
        Some(Statement::TSTypeAliasDeclaration(actual)),
    ) = (statements.next(), statements.next())
    else {
        return true;
    };
    // Unknown aliases, conditional/mapped types and recursive declarations are
    // not proof of a mismatch. The type checker owns their full evaluation.
    accepts(
        &expected.type_annotation,
        &actual.type_annotation,
        definitions,
        0,
    ) != Some(false)
}

fn accepts(
    expected: &TSType<'_>,
    actual: &TSType<'_>,
    definitions: &TypeDefinitions,
    depth: u8,
) -> Option<bool> {
    if depth >= 16 {
        return None;
    }
    match expected {
        TSType::TSAnyKeyword(_) | TSType::TSUnknownKeyword(_) => Some(true),
        TSType::TSNeverKeyword(_) => Some(false),
        TSType::TSParenthesizedType(inner) => {
            accepts(&inner.type_annotation, actual, definitions, depth + 1)
        }
        TSType::TSUnionType(union) => {
            let mut complete = true;
            for member in &union.types {
                match accepts(member, actual, definitions, depth + 1) {
                    Some(true) => return Some(true),
                    Some(false) => {}
                    None => complete = false,
                }
            }
            complete.then_some(false)
        }
        TSType::TSTypeReference(reference) if reference.type_arguments.is_none() => {
            let TSTypeName::IdentifierReference(name) = &reference.type_name else {
                return None;
            };
            let body = definitions.resolve(name.name.as_str())?;
            let source = cstr!("type __Alias = {body};");
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
            if parsed.panicked || !parsed.diagnostics.is_empty() {
                return None;
            }
            let Some(Statement::TSTypeAliasDeclaration(alias)) = parsed.program.body.first() else {
                return None;
            };
            accepts(&alias.type_annotation, actual, definitions, depth + 1)
        }
        TSType::TSStringKeyword(_) => Some(is_string(actual)),
        TSType::TSNumberKeyword(_) => Some(is_number(actual)),
        TSType::TSBooleanKeyword(_) => Some(is_boolean(actual)),
        TSType::TSLiteralType(expected) => match (&expected.literal, actual) {
            (TSLiteral::StringLiteral(left), TSType::TSLiteralType(right)) => Some(
                matches!(&right.literal, TSLiteral::StringLiteral(right) if left.value == right.value),
            ),
            (
                left @ (TSLiteral::NumericLiteral(_) | TSLiteral::UnaryExpression(_)),
                TSType::TSLiteralType(right),
            ) => numeric_value(left).map(|left| numeric_value(&right.literal) == Some(left)),
            (TSLiteral::BooleanLiteral(left), TSType::TSLiteralType(right)) => Some(
                matches!(&right.literal, TSLiteral::BooleanLiteral(right) if left.value == right.value),
            ),
            // A wide runtime type does not prove which literal was supplied.
            (TSLiteral::StringLiteral(_), _) if is_string(actual) => None,
            (TSLiteral::NumericLiteral(_) | TSLiteral::UnaryExpression(_), _)
                if is_number(actual) =>
            {
                None
            }
            (TSLiteral::BooleanLiteral(_), _) if is_boolean(actual) => None,
            (
                TSLiteral::StringLiteral(_)
                | TSLiteral::NumericLiteral(_)
                | TSLiteral::BooleanLiteral(_),
                _,
            ) => Some(false),
            _ => None,
        },
        // Actual values admitted by the caller are only scalar literals.
        TSType::TSNullKeyword(_)
        | TSType::TSUndefinedKeyword(_)
        | TSType::TSVoidKeyword(_)
        | TSType::TSObjectKeyword(_)
        | TSType::TSFunctionType(_)
        | TSType::TSConstructorType(_)
        | TSType::TSArrayType(_)
        | TSType::TSTupleType(_)
        | TSType::TSTypeLiteral(_) => Some(false),
        _ => None,
    }
}

fn numeric_value(literal: &TSLiteral<'_>) -> Option<f64> {
    match literal {
        TSLiteral::NumericLiteral(value) => Some(value.value),
        TSLiteral::UnaryExpression(unary) if unary.operator.as_str() == "-" => {
            match &unary.argument {
                Expression::NumericLiteral(value) => Some(-value.value),
                _ => None,
            }
        }
        _ => None,
    }
}

fn is_string(actual: &TSType<'_>) -> bool {
    matches!(actual, TSType::TSStringKeyword(_))
        || matches!(actual, TSType::TSLiteralType(literal) if matches!(literal.literal, TSLiteral::StringLiteral(_)))
}

fn is_number(actual: &TSType<'_>) -> bool {
    matches!(actual, TSType::TSNumberKeyword(_))
        || matches!(actual, TSType::TSLiteralType(literal) if matches!(literal.literal, TSLiteral::NumericLiteral(_) | TSLiteral::UnaryExpression(_)))
}

fn is_boolean(actual: &TSType<'_>) -> bool {
    matches!(actual, TSType::TSBooleanKeyword(_))
        || matches!(actual, TSType::TSLiteralType(literal) if matches!(literal.literal, TSLiteral::BooleanLiteral(_)))
}
