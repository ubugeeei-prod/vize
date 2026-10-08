//! Static `withDefaults` facts extracted from the authored object AST.

mod invalidation;

use oxc_ast::ast::{
    Argument, CallExpression, Expression, ObjectPropertyKind, PropertyKey, PropertyKind,
    VariableDeclarationKind,
};
use oxc_span::GetSpan;
use oxc_syntax::number::ToJsString;
use vize_carton::{CompactString, FxHashMap};

use super::super::ScriptParseResult;
use crate::macros::defaults::StaticDefaultObject;
use crate::macros::{MacroKind, MacroTracker};
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
    let keys = inline_owned_keys(result, call);
    let owner = if keys.is_empty() {
        None
    } else {
        result.macros.define_props().map(|owner| owner.start)
    };
    result
        .macros
        .set_with_defaults(expression, values, keys, owner);
}

fn inline_owned_keys(
    result: &ScriptParseResult,
    call: &CallExpression<'_>,
) -> FxHashMap<CompactString, Vec<(u32, u32)>> {
    let mut keys = FxHashMap::default();
    let Some(Argument::CallExpression(props)) = call.arguments.first() else {
        return keys;
    };
    if !matches!(&props.callee, Expression::Identifier(id)
        if MacroKind::from_name(id.name.as_str()) == Some(MacroKind::DefineProps))
        || !result
            .macros
            .define_props()
            .is_some_and(|owner| owner.start == props.span.start)
    {
        return keys;
    }
    let Some(Expression::ObjectExpression(object)) =
        call.arguments.get(1).and_then(Argument::as_expression)
    else {
        return keys;
    };
    // A separate object, spread, shorthand value or computed/accessor key has
    // a distinct owner or evaluation role. Its spelling is not ownership proof.
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return FxHashMap::default();
        };
        let PropertyKey::StaticIdentifier(key) = &property.key else {
            return FxHashMap::default();
        };
        if property.kind != PropertyKind::Init
            || property.computed
            || property.shorthand
            || property.method
        {
            return FxHashMap::default();
        }
        keys.entry(CompactString::new(key.name.as_str()))
            .or_insert_with(Vec::new)
            .push((key.span.start, key.span.end));
    }
    keys
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
