//! Options API computed/method bridge facts.

use oxc_ast::ast::{
    Argument, ArrayExpressionElement, ArrowFunctionExpression, CallExpression, Expression,
    Function, ObjectPropertyKind, Program,
};
use oxc_span::GetSpan;
use vize_carton::{String, append};

use super::options::{
    component_options_from_program, option_object_property, property_key_name, source_slice,
};
use super::{ScriptOptionsApiBridge, ScriptOptionsFunction, ScriptOptionsFunctionKind};

pub(super) fn options_api_bridge(
    program: &Program<'_>,
    source: &str,
) -> Option<ScriptOptionsApiBridge> {
    let options = component_options_from_program(program)?;
    let mut bridge = ScriptOptionsApiBridge::default();
    collect_function_bridge(
        source,
        options,
        "computed",
        ScriptOptionsFunctionKind::Computed,
        &mut bridge.computed,
        &mut bridge.mapped_types,
    );
    collect_function_bridge(
        source,
        options,
        "methods",
        ScriptOptionsFunctionKind::Method,
        &mut bridge.methods,
        &mut bridge.mapped_types,
    );
    Some(bridge)
}

fn collect_function_bridge(
    source: &str,
    options: &oxc_ast::ast::ObjectExpression<'_>,
    option_name: &str,
    kind: ScriptOptionsFunctionKind,
    output: &mut Vec<ScriptOptionsFunction>,
    mapped_types: &mut Vec<String>,
) {
    let Some(object) = option_object_property(options, option_name) else {
        return;
    };
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                if property.computed {
                    continue;
                }
                let Some(name) = property_key_name(&property.key) else {
                    continue;
                };
                if let Some(function) =
                    options_function_from_expression(source, name, &property.value, kind)
                {
                    output.push(function);
                }
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                if let Expression::CallExpression(call) = &spread.argument {
                    collect_mapped_type(call, mapped_types);
                }
            }
        }
    }
}

fn options_function_from_expression(
    source: &str,
    name: &str,
    expression: &Expression<'_>,
    kind: ScriptOptionsFunctionKind,
) -> Option<ScriptOptionsFunction> {
    let (params, body) = match expression {
        Expression::FunctionExpression(function) => function_parts(source, function)?,
        Expression::ArrowFunctionExpression(arrow) => arrow_function_parts(source, arrow)?,
        Expression::ParenthesizedExpression(value) => {
            return options_function_from_expression(source, name, &value.expression, kind);
        }
        Expression::TSAsExpression(value) => {
            return options_function_from_expression(source, name, &value.expression, kind);
        }
        Expression::TSSatisfiesExpression(value) => {
            return options_function_from_expression(source, name, &value.expression, kind);
        }
        Expression::TSNonNullExpression(value) => {
            return options_function_from_expression(source, name, &value.expression, kind);
        }
        _ => return None,
    };
    Some(ScriptOptionsFunction {
        kind,
        safe_name: safe_identifier(name),
        params,
        body,
    })
}

fn function_parts(source: &str, function: &Function<'_>) -> Option<(String, String)> {
    let params = params_source(source, &function.params)?;
    let body = function.body.as_ref()?;
    Some((
        params,
        String::from(source_slice(source, body.span())?.trim()),
    ))
}

fn arrow_function_parts(
    source: &str,
    arrow: &ArrowFunctionExpression<'_>,
) -> Option<(String, String)> {
    let params = params_source(source, &arrow.params)?;
    let body_source = source_slice(source, arrow.body.span())?.trim();
    if arrow.expression {
        let mut body = String::from("{ return ");
        body.push_str(body_source.trim_end_matches(';'));
        body.push_str("; }");
        Some((params, body))
    } else {
        Some((params, body_source.into()))
    }
}

fn params_source(source: &str, params: &oxc_ast::ast::FormalParameters<'_>) -> Option<String> {
    let mut result = String::default();
    let mut first = true;
    for param in &params.items {
        if !first {
            result.push_str(", ");
        }
        first = false;
        result.push_str(source_slice(source, param.span())?.trim());
    }
    if let Some(rest) = params.rest.as_ref() {
        if !first {
            result.push_str(", ");
        }
        result.push_str(source_slice(source, rest.span())?.trim());
    }
    Some(result)
}

fn collect_mapped_type(call: &CallExpression<'_>, mapped_types: &mut Vec<String>) {
    let Expression::Identifier(callee) = &call.callee else {
        return;
    };
    if !matches!(
        callee.name.as_str(),
        "mapState" | "mapGetters" | "mapWritableState" | "mapActions"
    ) {
        return;
    }
    let Some(Argument::Identifier(store)) = call.arguments.first() else {
        return;
    };
    let Some(Argument::ArrayExpression(keys)) = call.arguments.get(1) else {
        return;
    };
    let keys = keys
        .elements
        .iter()
        .filter_map(|element| {
            let ArrayExpressionElement::StringLiteral(literal) = element else {
                return None;
            };
            Some(literal.value.as_str())
        })
        .collect::<Vec<_>>();
    if keys.is_empty() {
        return;
    }
    let mut key_union = String::default();
    for (index, key) in keys.iter().enumerate() {
        if index > 0 {
            key_union.push_str(" | ");
        }
        append!(key_union, "'{key}'");
    }
    let mut mapped_type = String::default();
    append!(
        mapped_type,
        "[K in {key_union}]: ReturnType<typeof {}>[K]",
        store.name.as_str()
    );
    mapped_types.push(mapped_type);
}

fn safe_identifier(name: &str) -> String {
    let mut safe = String::with_capacity(name.len());
    for (index, character) in name.chars().enumerate() {
        let valid = if index == 0 {
            character.is_ascii_alphabetic() || character == '_' || character == '$'
        } else {
            character.is_ascii_alphanumeric() || character == '_' || character == '$'
        };
        safe.push(if valid { character } else { '_' });
    }
    if safe.is_empty() {
        safe.push('_');
    }
    safe
}
