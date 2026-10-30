//! Runtime constructor facts affect resolved template reads, never raw metadata.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKey};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashSet, String};
use vize_croquis::{BindingType, Croquis};

/// Vue resolves an absent runtime Boolean prop to `false`, including a
/// Boolean/String constructor union. A type annotation containing `boolean`
/// alone does not prove a Boolean constructor (`String as PropType<boolean>`).
pub(super) fn resolved_runtime_boolean_names(summary: &Croquis) -> FxHashSet<String> {
    let mut names = FxHashSet::default();
    let Some(call) = summary
        .macros
        .define_props()
        .filter(|call| call.type_args.is_none())
    else {
        return names;
    };
    let Some(source) = call.runtime_args.as_deref() else {
        return names;
    };
    // An authored local constructor of this name is not the global Boolean.
    if crate::virtual_ts::script_facts::binding_type(summary, "Boolean")
        .is_some_and(|binding| !matches!(binding, BindingType::Props))
    {
        return names;
    }
    let allocator = Allocator::default();
    let Ok(expression) = Parser::new(&allocator, source, SourceType::ts()).parse_expression()
    else {
        return names;
    };
    let Expression::ObjectExpression(object) = unwrapped(&expression) else {
        return names;
    };
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            // A spread can replace any preceding declaration.
            names.clear();
            continue;
        };
        if property.computed {
            names.clear();
            continue;
        }
        let Some(name) = key_name(&property.key) else {
            continue;
        };
        names.remove(name);
        if declares_boolean(&property.value) {
            names.insert(name.into());
        }
    }
    names
}

fn unwrapped<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    match expression {
        Expression::TSAsExpression(value) => unwrapped(&value.expression),
        Expression::TSSatisfiesExpression(value) => unwrapped(&value.expression),
        Expression::TSNonNullExpression(value) => unwrapped(&value.expression),
        Expression::ParenthesizedExpression(value) => unwrapped(&value.expression),
        expression => expression,
    }
}

fn declares_boolean(expression: &Expression<'_>) -> bool {
    match unwrapped(expression) {
        Expression::Identifier(identifier) => identifier.name == "Boolean",
        Expression::ArrayExpression(array) => array
            .elements
            .iter()
            .any(|element| element.as_expression().is_some_and(declares_boolean)),
        Expression::ObjectExpression(object) => {
            // Unresolved option spreads/computed keys can supply an explicit
            // `default: undefined`; they do not prove absent-prop resolution.
            if object.properties.iter().any(|property| {
                !matches!(property, ObjectPropertyKind::ObjectProperty(property) if !property.computed)
            }) {
                return false;
            }
            object
                .properties
                .iter()
                .rev()
                .find_map(|property| {
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        return None;
                    };
                    (key_name(&property.key) == Some("type"))
                        .then(|| declares_boolean(&property.value))
                })
                .unwrap_or(false)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests;

fn key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(value) => Some(value.value.as_str()),
        _ => None,
    }
}
