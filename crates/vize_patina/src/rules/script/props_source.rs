//! Shared resolution of runtime prop declarations for the `script/*` prop
//! rules.
//!
//! Vue declares runtime props in two places that these rules treat uniformly:
//!
//! * the Options API `props` option
//!   (`export default { props: ... }` / `defineComponent({ props: ... })` /
//!   a same-file identifier bound to such an object), and
//! * the `<script setup>` runtime macro `defineProps({ ... })` /
//!   `defineProps([ ... ])`.
//!
//! Both accept the same two shapes — an **object** form
//! (`{ name: <type-or-descriptor> }`) and an **array** form (`['name']`) — so
//! this module yields a [`PropsSource`] for each declaration found, exposing its
//! object entries (as [`PropDescriptor`]s) and its array name literals.
//!
//! The type-based `defineProps<{ ... }>()` form carries no runtime descriptor
//! (no `default`/`required`/`type` members) and is intentionally **not**
//! collected here.

use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, ExportDefaultDeclarationKind, Expression,
    ObjectExpression, ObjectPropertyKind, Program, PropertyKey, Statement, StringLiteral,
};

use vize_carton::FxHashMap;

/// A single object-form prop entry: its name, the key node (for span
/// reporting), and its value (a type shorthand or a descriptor object).
#[derive(Clone, Copy)]
pub(super) struct PropDescriptor<'a> {
    pub(super) name: &'a str,
    pub(super) key: &'a PropertyKey<'a>,
    pub(super) value: &'a Expression<'a>,
}

/// One resolved runtime props declaration (object or array form).
pub(super) enum PropsSource<'a> {
    /// Object form: `props: { ... }` / `defineProps({ ... })`.
    Object(&'a ObjectExpression<'a>),
    /// Array form: `props: ['a']` / `defineProps(['a'])`.
    Array(&'a oxc_ast::ast::ArrayExpression<'a>),
}

impl<'a> PropsSource<'a> {
    /// The object-form prop entries, or an empty iterator for the array form.
    /// Computed and spread members are skipped.
    pub(super) fn object_props(&self) -> impl Iterator<Item = PropDescriptor<'a>> + '_ {
        let object = match self {
            PropsSource::Object(object) => Some(*object),
            PropsSource::Array(_) => None,
        };
        object
            .into_iter()
            .flat_map(|object| object.properties.iter())
            .filter_map(|property| {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    return None;
                };
                if property.computed {
                    return None;
                }
                let name = property_key_name(&property.key)?;
                Some(PropDescriptor {
                    name,
                    key: &property.key,
                    value: &property.value,
                })
            })
    }

    /// The array-form name string-literals, or an empty iterator for the object
    /// form.
    pub(super) fn array_names(&self) -> impl Iterator<Item = &'a StringLiteral<'a>> + '_ {
        let array = match self {
            PropsSource::Array(array) => Some(*array),
            PropsSource::Object(_) => None,
        };
        array
            .into_iter()
            .flat_map(|array| array.elements.iter())
            .filter_map(|element| match element.as_expression() {
                Some(Expression::StringLiteral(literal)) => Some(literal.as_ref()),
                _ => None,
            })
    }
}

/// Collect every runtime props declaration in the program: the Options API
/// `props` option (if any) and every top-level `defineProps(...)` runtime call.
pub(super) fn collect_runtime_props<'a>(program: &'a Program<'a>) -> Vec<PropsSource<'a>> {
    let mut sources = Vec::new();
    let bindings = collect_object_bindings(program);

    if let Some(options) = resolve_options_object(program, &bindings)
        && let Some(props) = option_props_source(options, &bindings)
    {
        sources.push(props);
    }

    collect_define_props_sources(program, &mut sources);

    sources
}

/// Append the runtime argument of each top-level `defineProps(...)` call.
fn collect_define_props_sources<'a>(program: &'a Program<'a>, sources: &mut Vec<PropsSource<'a>>) {
    for statement in &program.body {
        for call in top_level_calls(statement) {
            let Expression::Identifier(callee) = &call.callee else {
                continue;
            };
            if callee.name.as_str() != "defineProps" {
                continue;
            }
            if let Some(source) = call
                .arguments
                .first()
                .and_then(Argument::as_expression)
                .and_then(props_source_from_expression)
            {
                sources.push(source);
            }
        }
    }
}

/// Interpret an expression as a props source (object or array literal),
/// unwrapping TS-only wrappers and parentheses.
fn props_source_from_expression<'a>(expression: &'a Expression<'a>) -> Option<PropsSource<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(PropsSource::Object(object)),
        Expression::ArrayExpression(array) => Some(PropsSource::Array(array)),
        Expression::ParenthesizedExpression(paren) => {
            props_source_from_expression(&paren.expression)
        }
        Expression::TSAsExpression(ts) => props_source_from_expression(&ts.expression),
        Expression::TSSatisfiesExpression(ts) => props_source_from_expression(&ts.expression),
        Expression::TSNonNullExpression(ts) => props_source_from_expression(&ts.expression),
        _ => None,
    }
}

/// The `props` option's value as a props source, resolving an identifier-bound
/// object when the value is a bare identifier.
fn option_props_source<'a>(
    options: &'a ObjectExpression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<PropsSource<'a>> {
    options.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed || property_key_name(&property.key) != Some("props") {
            return None;
        }
        if let Expression::Identifier(identifier) = &property.value {
            return bindings
                .get(identifier.name.as_str())
                .copied()
                .map(PropsSource::Object);
        }
        props_source_from_expression(&property.value)
    })
}

// ---------------------------------------------------------------------------
// Options-object resolution (export default / defineComponent / identifier).
// A self-contained subset mirroring `no_required_prop_with_default`.
// ---------------------------------------------------------------------------

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

fn resolve_options_object<'a>(
    program: &'a Program<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    program.body.iter().find_map(|statement| {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            return None;
        };
        match &export.declaration {
            ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object.as_ref()),
            ExportDefaultDeclarationKind::CallExpression(call) => {
                component_options_from_call(call, bindings)
            }
            ExportDefaultDeclarationKind::Identifier(identifier) => {
                bindings.get(identifier.name.as_str()).copied()
            }
            declaration => declaration
                .as_expression()
                .and_then(|expression| resolve_object_or_binding(expression, bindings)),
        }
    })
}

fn component_options_from_call<'a>(
    call: &'a CallExpression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    if !is_define_component_callee(&call.callee) {
        return None;
    }
    match call.arguments.first()? {
        Argument::ObjectExpression(object) => Some(object.as_ref()),
        Argument::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        argument => argument
            .as_expression()
            .and_then(|expression| resolve_object_or_binding(expression, bindings)),
    }
}

fn is_define_component_callee(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::Identifier(identifier) => matches!(
            identifier.name.as_str(),
            "defineComponent" | "_defineComponent"
        ),
        Expression::StaticMemberExpression(member) => matches!(
            member.property.name.as_str(),
            "defineComponent" | "_defineComponent"
        ),
        _ => false,
    }
}

fn resolve_object_or_binding<'a>(
    expression: &'a Expression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    if let Expression::Identifier(identifier) = expression {
        return bindings.get(identifier.name.as_str()).copied();
    }
    unwrap_object_expression(expression)
}

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

// ---------------------------------------------------------------------------
// Shared small helpers.
// ---------------------------------------------------------------------------

/// Collect the call expressions at the top level of a single statement: a bare
/// `defineProps(...)` expression statement, or the initializer of
/// `const props = defineProps(...)` (including a destructured
/// `const { x } = defineProps(...)`).
fn top_level_calls<'a, 'b>(statement: &'b Statement<'a>) -> Vec<&'b CallExpression<'a>> {
    match statement {
        Statement::ExpressionStatement(expression_statement) => {
            unwrap_call(&expression_statement.expression)
                .into_iter()
                .collect()
        }
        Statement::VariableDeclaration(declaration) => declaration
            .declarations
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .filter_map(unwrap_call)
            .collect(),
        _ => Vec::new(),
    }
}

fn unwrap_call<'a, 'b>(expression: &'b Expression<'a>) -> Option<&'b CallExpression<'a>> {
    match expression {
        Expression::CallExpression(call) => Some(call),
        Expression::ParenthesizedExpression(paren) => unwrap_call(&paren.expression),
        Expression::TSAsExpression(ts) => unwrap_call(&ts.expression),
        Expression::TSSatisfiesExpression(ts) => unwrap_call(&ts.expression),
        Expression::TSNonNullExpression(ts) => unwrap_call(&ts.expression),
        _ => None,
    }
}

pub(super) fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}
