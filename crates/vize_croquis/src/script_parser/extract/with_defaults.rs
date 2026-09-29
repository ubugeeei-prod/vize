//! Static `withDefaults` facts extracted from the authored object AST.

mod invalidation;

use oxc_ast::ast::{
    CallExpression, Expression, ObjectPropertyKind, PropertyKey, PropertyKind,
    VariableDeclarationKind,
};
use oxc_span::GetSpan;
use oxc_syntax::number::ToJsString;
use vize_carton::CompactString;

use super::super::ScriptParseResult;
use crate::macros::MacroTracker;
use crate::macros::defaults::StaticDefaultObject;
use crate::scope::ScopeKind;

pub(in crate::script_parser) use invalidation::{
    invalidate_default_expression, invalidate_default_objects,
};

pub(super) fn record_defaults(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
    source: &str,
) {
    let Some(argument) = call.arguments.get(1) else {
        return;
    };
    let expression = CompactString::new(argument.span().source_text(source));
    let values = argument
        .as_expression()
        .and_then(|expression| collect_object(result, expression, source))
        .unwrap_or_default()
        .values;
    result.macros.set_with_defaults(expression, values);
}

pub(super) fn record_object_binding(
    result: &mut ScriptParseResult,
    name: &str,
    expression: &Expression<'_>,
    kind: VariableDeclarationKind,
    source: &str,
) {
    let object = (kind == VariableDeclarationKind::Const
        && matches!(
            result.scopes.current_scope().kind,
            ScopeKind::ScriptSetup | ScopeKind::NonScriptSetup | ScopeKind::Module
        ))
    .then(|| collect_object(result, expression, source))
    .flatten();
    result.macros.record_default_object(name, object);
}

fn collect_object(
    result: &ScriptParseResult,
    expression: &Expression<'_>,
    source: &str,
) -> Option<StaticDefaultObject> {
    let object = match expression.get_inner_expression() {
        Expression::ObjectExpression(object) => object,
        Expression::Identifier(identifier) => {
            return result
                .macros
                .default_object(identifier.name.as_str())
                .cloned();
        }
        _ => return None,
    };
    let mut output = StaticDefaultObject::default();
    for property in &object.properties {
        match property {
            ObjectPropertyKind::SpreadProperty(spread) => {
                match collect_object(result, &spread.argument, source) {
                    Some(object) => output.spread(object),
                    None => output.clear(),
                }
            }
            ObjectPropertyKind::ObjectProperty(property) => {
                // Reading defaults or spreading this object can execute an
                // accessor. Its effects are not initializer facts.
                if property.kind != PropertyKind::Init {
                    return None;
                }
                let name = match &property.key {
                    PropertyKey::StaticIdentifier(id) if !property.computed => {
                        CompactString::new(id.name.as_str())
                    }
                    PropertyKey::StringLiteral(literal) => {
                        CompactString::new(literal.value.as_str())
                    }
                    PropertyKey::NumericLiteral(literal) => {
                        CompactString::new(literal.value.to_js_string())
                    }
                    _ => {
                        output.clear();
                        continue;
                    }
                };
                // `removed` is copied only from another object's `removed`,
                // which starts empty, so deleting here would hash an empty set.
                output.values.insert(
                    name,
                    CompactString::new(property.value.span().source_text(source)),
                );
            }
        }
    }
    Some(output)
}

fn spread_is_getter_free(macros: &MacroTracker, expression: &Expression<'_>) -> bool {
    match expression.get_inner_expression() {
        Expression::Identifier(identifier) => {
            macros.default_object(identifier.name.as_str()).is_some()
        }
        Expression::ObjectExpression(object) => {
            object.properties.iter().all(|property| match property {
                ObjectPropertyKind::ObjectProperty(property) => property.kind == PropertyKind::Init,
                ObjectPropertyKind::SpreadProperty(spread) => {
                    spread_is_getter_free(macros, &spread.argument)
                }
            })
        }
        _ => false,
    }
}
