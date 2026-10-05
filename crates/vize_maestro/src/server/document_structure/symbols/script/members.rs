//! Static object/class members retain authored key and complete member spans.

use oxc_ast::ast::{Class, ClassElement, Expression, ObjectPropertyKind, PropertyKey};
use oxc_span::GetSpan;
use tower_lsp::lsp_types::{DocumentSymbol, SymbolKind};
use vize_l0::line_index::LineIndex;

pub(super) fn unwrapped<'a>(value: &'a Expression<'a>) -> &'a Expression<'a> {
    match value {
        Expression::TSAsExpression(value) => unwrapped(&value.expression),
        Expression::TSSatisfiesExpression(value) => unwrapped(&value.expression),
        Expression::TSNonNullExpression(value) => unwrapped(&value.expression),
        Expression::ParenthesizedExpression(value) => unwrapped(&value.expression),
        _ => value,
    }
}

fn name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(value) => Some(value.value.as_str()),
        _ => None,
    }
}

pub(super) fn object_children(
    value: &Expression<'_>,
    index: &LineIndex<'_>,
    base: usize,
) -> Vec<DocumentSymbol> {
    let Expression::ObjectExpression(object) = unwrapped(value) else {
        return Vec::new();
    };
    object
        .properties
        .iter()
        .filter_map(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return None;
            };
            if property.computed {
                return None;
            }
            let name = name(&property.key)?;
            let value = unwrapped(&property.value);
            let kind = if property.method
                || matches!(
                    value,
                    Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
                ) {
                SymbolKind::METHOD
            } else {
                SymbolKind::PROPERTY
            };
            Some(super::super::symbol(
                name,
                kind,
                index,
                base,
                property.span,
                property.key.span(),
                object_children(value, index, base),
            ))
        })
        .collect()
}

pub(super) fn class_children(
    class: &Class<'_>,
    index: &LineIndex<'_>,
    base: usize,
) -> Vec<DocumentSymbol> {
    class
        .body
        .body
        .iter()
        .filter_map(|member| {
            let (key, span, computed, kind) = match member {
                ClassElement::MethodDefinition(method) => (
                    &method.key,
                    method.span,
                    method.computed,
                    SymbolKind::METHOD,
                ),
                ClassElement::PropertyDefinition(property) => (
                    &property.key,
                    property.span,
                    property.computed,
                    SymbolKind::FIELD,
                ),
                ClassElement::AccessorProperty(property) => (
                    &property.key,
                    property.span,
                    property.computed,
                    SymbolKind::PROPERTY,
                ),
                _ => return None,
            };
            if computed {
                return None;
            }
            Some(super::super::symbol(
                name(key)?,
                kind,
                index,
                base,
                span,
                key.span(),
                Vec::new(),
            ))
        })
        .collect()
}
