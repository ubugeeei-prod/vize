//! script/no-required-prop-with-default
//!
//! Disallow a prop that is both `required: true` and has a `default`.
//!
//! A `required` prop is always provided by the parent, so its `default` value
//! can never apply. Declaring both is contradictory and the default is dead
//! code — almost always a sign that one of the two was meant to be removed.
//!
//! Covers the Options API object form
//! (`props: { x: { required: true, default: ... } }`), including
//! `defineComponent({...})` and same-file identifier-bound options/props
//! objects. `required: false` and a non-literal `required` value are ignored.

use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, ExportDefaultDeclarationKind, Expression,
    ObjectExpression, ObjectPropertyKind, Program, PropertyKey, Statement,
};
use oxc_span::GetSpan;

use vize_carton::{CompactString, FxHashMap};

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-required-prop-with-default",
    description: "Disallow a prop that is both required: true and has a default",
    default_severity: Severity::Error,
};

/// Disallow a prop declared with both `required: true` and a `default`.
pub struct NoRequiredPropWithDefault;

impl ScriptRule for NoRequiredPropWithDefault {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let object_bindings = collect_object_bindings(program);
        let Some(options) = resolve_options_object(program, &object_bindings) else {
            return;
        };
        let Some(props) = option_object_property(options, "props", &object_bindings) else {
            return;
        };
        check_props_object(props, offset, result);
    }
}

/// Report every prop whose declaration object has both `required: true` and a
/// `default` property.
fn check_props_object(props: &ObjectExpression<'_>, offset: usize, result: &mut ScriptLintResult) {
    for property in &props.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        let (Some(name), Expression::ObjectExpression(declaration)) =
            (property_key_name(&property.key), &property.value)
        else {
            continue;
        };

        if let (Some(required_span), Some(default_span)) = (
            prop_required_true_span(declaration),
            prop_member_span(declaration, "default"),
        ) {
            report(
                name,
                &property.key,
                required_span,
                default_span,
                offset,
                result,
            );
        }
    }
}

/// Span of a `required: true` member (literal boolean `true` only), if present.
fn prop_required_true_span(declaration: &ObjectExpression<'_>) -> Option<(u32, u32)> {
    declaration.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property_key_name(&property.key) != Some("required") {
            return None;
        }
        let Expression::BooleanLiteral(boolean) = &property.value else {
            return None;
        };
        boolean
            .value
            .then(|| (property.key.span().start, property.value.span().end))
    })
}

/// Span (key..value) of a named member, if present.
fn prop_member_span(declaration: &ObjectExpression<'_>, key: &str) -> Option<(u32, u32)> {
    declaration.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property_key_name(&property.key) != Some(key) {
            return None;
        }
        Some((property.key.span().start, property.value.span().end))
    })
}

fn report(
    name: &str,
    key: &PropertyKey<'_>,
    required_span: (u32, u32),
    default_span: (u32, u32),
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let key_span = key.span();
    let base = offset as u32;

    let mut message = CompactString::with_capacity(name.len() + 56);
    message.push_str("Prop '");
    message.push_str(name);
    message.push_str("' is required but also declares a default");

    let diagnostic = LintDiagnostic::error(
        META.name,
        message,
        base + key_span.start,
        base + key_span.end,
    )
    .with_label(
        "marked required here",
        base + required_span.0,
        base + required_span.1,
    )
    .with_label(
        "default declared here",
        base + default_span.0,
        base + default_span.1,
    )
    .with_help(
        "A required prop is always provided, so its default is unreachable; \
                 drop `required: true` or remove the default.",
    );
    result.add_diagnostic(diagnostic);
}

// Options-object resolution: a small, self-contained subset of the croquis
// Options API resolution (object / defineComponent / paren / as / satisfies /
// non-null / same-file identifier binding).

/// Map of `const x = { ... }` identifier bindings to their object literal.
fn collect_object_bindings<'a>(
    program: &'a Program<'a>,
) -> FxHashMap<&'a str, &'a ObjectExpression<'a>> {
    let mut bindings = FxHashMap::default();
    for statement in program.body.iter() {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in declaration.declarations.iter() {
            let BindingPattern::BindingIdentifier(id) = &declarator.id else {
                continue;
            };
            if let Some(object) = declarator.init.as_ref().and_then(unwrap_object_expression) {
                bindings.entry(id.name.as_str()).or_insert(object);
            }
        }
    }
    bindings
}

/// Resolve the Options API options object from `export default`.
fn resolve_options_object<'a>(
    program: &'a Program<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    program.body.iter().find_map(|statement| {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            return None;
        };
        match &export.declaration {
            ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object.as_ref()),
            ExportDefaultDeclarationKind::CallExpression(call) => {
                component_options_from_call(call, object_bindings)
            }
            ExportDefaultDeclarationKind::Identifier(identifier) => {
                object_bindings.get(identifier.name.as_str()).copied()
            }
            declaration => declaration
                .as_expression()
                .and_then(|expression| resolve_object_or_binding(expression, object_bindings)),
        }
    })
}

fn component_options_from_call<'a>(
    call: &'a CallExpression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    if !is_define_component_callee(&call.callee) {
        return None;
    }
    match call.arguments.first()? {
        Argument::ObjectExpression(object) => Some(object.as_ref()),
        Argument::Identifier(identifier) => object_bindings.get(identifier.name.as_str()).copied(),
        argument => argument
            .as_expression()
            .and_then(|expression| resolve_object_or_binding(expression, object_bindings)),
    }
}

fn is_define_component_callee(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::Identifier(identifier) => {
            matches!(
                identifier.name.as_str(),
                "defineComponent" | "_defineComponent"
            )
        }
        Expression::StaticMemberExpression(member) => {
            matches!(
                member.property.name.as_str(),
                "defineComponent" | "_defineComponent"
            )
        }
        _ => false,
    }
}

/// Value of a named option property as an object literal (resolving a same-file
/// identifier binding when the value is a bare identifier).
fn option_object_property<'a>(
    object: &'a ObjectExpression<'a>,
    key_name: &str,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property_key_name(&property.key) != Some(key_name) {
            return None;
        }
        resolve_object_or_binding(&property.value, object_bindings)
    })
}

fn resolve_object_or_binding<'a>(
    expression: &'a Expression<'a>,
    object_bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    if let Expression::Identifier(identifier) = expression {
        return object_bindings.get(identifier.name.as_str()).copied();
    }
    unwrap_object_expression(expression)
}

/// Unwrap paren / `as` / `satisfies` / non-null wrappers down to a plain object
/// literal.
fn unwrap_object_expression<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object.as_ref()),
        Expression::ParenthesizedExpression(paren) => unwrap_object_expression(&paren.expression),
        Expression::TSAsExpression(ts) => unwrap_object_expression(&ts.expression),
        Expression::TSSatisfiesExpression(ts) => unwrap_object_expression(&ts.expression),
        Expression::TSNonNullExpression(ts) => unwrap_object_expression(&ts.expression),
        _ => None,
    }
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
