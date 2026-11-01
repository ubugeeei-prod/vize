use oxc_ast::ast::{Argument, CallExpression, Expression, ObjectExpression, ObjectPropertyKind};
use oxc_span::GetSpan;
use vize_carton::{FxHashMap, ToCompactString};
use vize_croquis::macros::PropDefinition;

use crate::batch::virtual_project::setup_props::syntax::{
    runtime_call_name, runtime_object_property_name, runtime_prop_shape_member_type,
};

pub(super) fn is_runtime_props_wrapper(call: &CallExpression<'_>) -> bool {
    match &call.callee {
        Expression::Identifier(identifier) => {
            matches!(identifier.name.as_str(), "buildProps" | "defineProps")
        }
        _ => false,
    }
}

pub(super) fn call_expression_name<'a>(call: &'a CallExpression<'a>) -> Option<&'a str> {
    super::super::syntax::call_expression_name(call)
}

/// Whether a `default` relaxes the prop's `required: true` for the parent: Vue
/// publishes every defaulted prop as optional (`Partial<Defaults>`), except a
/// plain `Boolean` prop, whose `required: true` outranks its default.
pub(super) fn resolve_runtime_prop_options<'a>(
    expr: &'a Expression<'a>,
    source: &str,
    local_values: &FxHashMap<&'a str, &'a Expression<'a>>,
) -> ResolvedRuntimePropOptions {
    let Some(object) = runtime_prop_options_object(expr) else {
        return ResolvedRuntimePropOptions {
            required: false,
            default_relaxes_required: false,
            defaulted: runtime_call_name(expr).is_some_and(|name| name.ends_with("WithDefault")),
            plain_boolean: false,
        };
    };
    let mut options = RuntimePropOptionsState::default();
    collect_runtime_prop_options(object, source, local_values, &mut Vec::new(), &mut options);
    ResolvedRuntimePropOptions {
        required: options.required == Some(true),
        default_relaxes_required: options.has_default == Some(true)
            && options.plain_boolean == Some(false),
        defaulted: options.has_default == Some(true)
            || options.declares_boolean == Some(true)
            || options.has_resolved_default_marker,
        plain_boolean: options.plain_boolean == Some(true),
    }
}

pub(super) struct ResolvedRuntimePropOptions {
    pub(super) required: bool,
    pub(super) default_relaxes_required: bool,
    pub(super) defaulted: bool,
    pub(super) plain_boolean: bool,
}

#[derive(Debug)]
struct RuntimePropOptionsState {
    /// `None` means an unresolved spread may override this option.
    required: Option<bool>,
    /// `None` means an unresolved spread may contribute this option.
    has_default: Option<bool>,
    /// `None` means an unresolved spread may override the effective `type`.
    plain_boolean: Option<bool>,
    /// `None` means an unresolved spread may override the effective `type`.
    declares_boolean: Option<bool>,
    has_resolved_default_marker: bool,
}

impl Default for RuntimePropOptionsState {
    fn default() -> Self {
        Self {
            required: Some(false),
            has_default: Some(false),
            plain_boolean: Some(false),
            declares_boolean: Some(false),
            has_resolved_default_marker: false,
        }
    }
}

fn collect_runtime_prop_options<'a>(
    object: &'a ObjectExpression<'a>,
    source: &str,
    local_values: &FxHashMap<&'a str, &'a Expression<'a>>,
    resolving: &mut Vec<&'a str>,
    options: &mut RuntimePropOptionsState,
) {
    for property in &object.properties {
        match property {
            ObjectPropertyKind::ObjectProperty(property) => {
                match runtime_object_property_name(&property.key) {
                    Some("required") => {
                        options.required = match &property.value {
                            Expression::BooleanLiteral(value) => Some(value.value),
                            _ => None,
                        };
                    }
                    Some("default") => options.has_default = Some(true),
                    Some("type") => {
                        options.plain_boolean =
                            Some(runtime_prop_is_plain_boolean_constructor(&property.value));
                        options.declares_boolean =
                            Some(runtime_prop_declares_boolean(&property.value));
                    }
                    _ => {}
                }
                options.has_resolved_default_marker |= property
                    .key
                    .span()
                    .source_text(source)
                    .trim()
                    .ends_with("WithDefault");
            }
            ObjectPropertyKind::SpreadProperty(spread) => collect_runtime_prop_options_from_expr(
                &spread.argument,
                source,
                local_values,
                resolving,
                options,
            ),
        }
    }
}

fn collect_runtime_prop_options_from_expr<'a>(
    expr: &'a Expression<'a>,
    source: &str,
    local_values: &FxHashMap<&'a str, &'a Expression<'a>>,
    resolving: &mut Vec<&'a str>,
    options: &mut RuntimePropOptionsState,
) {
    match expr {
        Expression::ObjectExpression(object) => {
            collect_runtime_prop_options(object, source, local_values, resolving, options);
        }
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            let Some(value) = local_values.get(name) else {
                mark_runtime_prop_options_unknown(options);
                return;
            };
            if resolving.contains(&name) {
                mark_runtime_prop_options_unknown(options);
                return;
            }
            resolving.push(name);
            collect_runtime_prop_options_from_expr(value, source, local_values, resolving, options);
            resolving.pop();
        }
        Expression::TSAsExpression(ts_as) => collect_runtime_prop_options_from_expr(
            &ts_as.expression,
            source,
            local_values,
            resolving,
            options,
        ),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            collect_runtime_prop_options_from_expr(
                &ts_satisfies.expression,
                source,
                local_values,
                resolving,
                options,
            );
        }
        Expression::TSNonNullExpression(ts_non_null) => collect_runtime_prop_options_from_expr(
            &ts_non_null.expression,
            source,
            local_values,
            resolving,
            options,
        ),
        Expression::ParenthesizedExpression(parenthesized) => {
            collect_runtime_prop_options_from_expr(
                &parenthesized.expression,
                source,
                local_values,
                resolving,
                options,
            );
        }
        Expression::CallExpression(_) => {
            options.has_resolved_default_marker |=
                runtime_call_name(expr).is_some_and(|name| name.ends_with("WithDefault"));
            mark_runtime_prop_options_unknown(options);
        }
        _ => mark_runtime_prop_options_unknown(options),
    }
}

fn mark_runtime_prop_options_unknown(options: &mut RuntimePropOptionsState) {
    options.required = None;
    if options.has_default != Some(true) {
        options.has_default = None;
    }
    options.plain_boolean = None;
    options.declares_boolean = None;
}

fn runtime_prop_is_plain_boolean_constructor(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Identifier(identifier) => identifier.name.as_str() == "Boolean",
        Expression::TSAsExpression(ts_as) => {
            runtime_prop_is_plain_boolean_constructor(&ts_as.expression)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            runtime_prop_is_plain_boolean_constructor(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            runtime_prop_is_plain_boolean_constructor(&ts_non_null.expression)
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            runtime_prop_is_plain_boolean_constructor(&parenthesized.expression)
        }
        _ => false,
    }
}

pub(super) fn collect_props_from_string_array_helper_call(
    call: &CallExpression<'_>,
    root_runtime_binding: &str,
    props: &mut Vec<PropDefinition>,
) {
    let Expression::Identifier(callee) = &call.callee else {
        return;
    };
    if callee.name.as_str() != "useAriaProps" {
        return;
    }
    let Some(Argument::ArrayExpression(names)) = call.arguments.first() else {
        return;
    };
    for element in &names.elements {
        let oxc_ast::ast::ArrayExpressionElement::StringLiteral(name) = element else {
            continue;
        };
        let name = name.value.as_str();
        props.push(PropDefinition {
            name: name.to_compact_string(),
            prop_type: runtime_prop_shape_member_type(root_runtime_binding, name),
            required: false,
            default_value: None,
        });
    }
}

fn runtime_prop_options_object<'a>(expr: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expr {
        Expression::ObjectExpression(object) => Some(object),
        Expression::TSAsExpression(ts_as) => runtime_prop_options_object(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            runtime_prop_options_object(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            runtime_prop_options_object(&ts_non_null.expression)
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            runtime_prop_options_object(&parenthesized.expression)
        }
        _ => None,
    }
}

fn runtime_prop_declares_boolean(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::Identifier(identifier) => identifier.name.as_str() == "Boolean",
        Expression::ArrayExpression(array) => array.elements.iter().any(|element| match element {
            oxc_ast::ast::ArrayExpressionElement::Identifier(identifier) => {
                identifier.name.as_str() == "Boolean"
            }
            oxc_ast::ast::ArrayExpressionElement::SpreadElement(spread) => {
                runtime_prop_declares_boolean(&spread.argument)
            }
            _ => false,
        }),
        Expression::ObjectExpression(object) => object.properties.iter().any(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return false;
            };
            runtime_object_property_name(&property.key) == Some("type")
                && runtime_prop_declares_boolean(&property.value)
        }),
        Expression::TSAsExpression(ts_as) => runtime_prop_declares_boolean(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            runtime_prop_declares_boolean(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            runtime_prop_declares_boolean(&ts_non_null.expression)
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            runtime_prop_declares_boolean(&parenthesized.expression)
        }
        _ => false,
    }
}
