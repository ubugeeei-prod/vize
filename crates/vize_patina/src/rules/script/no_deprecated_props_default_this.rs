//! script/no-deprecated-props-default-this
//!
//! Disallow accessing `this` inside a prop `default` or `validator` function.
//! In Vue 2 these functions ran with `this` bound to the (pre-creation)
//! component instance, so deriving a default via `this.*` was common. In Vue 3
//! they are called as plain functions, so `this` is `undefined` and any
//! `this.*` access is a bug; Vue 3 passes the raw props as the first argument
//! to the `default` factory instead.
//!
//! This is a Vue 2 -> 3 migration rule, scoped to the Options API object-form
//! `props` option. For each object-form prop, the `default` and `validator`
//! functions (regular or arrow) are inspected, and a `this.*` reference at the
//! function's own `this`-scope is reported; nested regular functions (which
//! rebind `this`) are not traversed.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   props: {
//!     size: {
//!       type: Number,
//!       // `this` is not the component instance in Vue 3.
//!       default() {
//!         return this.defaultSize
//!       }
//!     },
//!     value: {
//!       type: Number,
//!       validator() {
//!         return this.value > 0
//!       }
//!     }
//!   }
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   props: {
//!     size: {
//!       type: Number,
//!       // Vue 3 passes the raw props as the first argument instead.
//!       default(props) {
//!         return props.baseSize
//!       }
//!     },
//!     value: {
//!       type: Number,
//!       validator(value) {
//!         return value > 0
//!       }
//!     }
//!   }
//! }
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, ExportDefaultDeclarationKind, Expression, Function,
    ObjectExpression, ObjectProperty, ObjectPropertyKind, Program, PropertyKey, Statement,
    ThisExpression,
};
use oxc_ast_visit::Visit;
use oxc_span::Span;
use vize_carton::FxHashMap;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-deprecated-props-default-this",
    description: "Disallow `this` inside a prop default/validator function (removed in Vue 3)",
    default_severity: Severity::Error,
};

/// The prop-declaration properties whose function value used to receive the
/// component instance as `this` in Vue 2.
const THIS_AWARE_PROP_FUNCTIONS: &[&str] = &["default", "validator"];

/// Disallow `this` inside a prop `default`/`validator` function.
pub struct NoDeprecatedPropsDefaultThis;

impl ScriptRule for NoDeprecatedPropsDefaultThis {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let Some(options) = find_component_options(program) else {
            return;
        };
        let Some(props) = find_props_object(options) else {
            return;
        };
        check_props_object(props, offset, result);
    }
}

fn check_props_object(props: &ObjectExpression<'_>, offset: usize, result: &mut ScriptLintResult) {
    for property in &props.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        // Each prop declaration must itself be an object to carry a
        // `default`/`validator`; shorthand `foo: Number` forms have none.
        let Expression::ObjectExpression(declaration) = &property.value else {
            continue;
        };
        check_prop_declaration(declaration, offset, result);
    }
}

fn check_prop_declaration(
    declaration: &ObjectExpression<'_>,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    for property in &declaration.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        let Some(name) = property_key_name(&property.key) else {
            continue;
        };
        if !THIS_AWARE_PROP_FUNCTIONS.contains(&name) {
            continue;
        }
        check_prop_function(property, offset, result);
    }
}

/// Walk the `default`/`validator` function body and report any `this` access at
/// the function's own `this`-scope.
fn check_prop_function(
    property: &ObjectProperty<'_>,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let mut visitor = ThisVisitor { offset, result };
    match &property.value {
        // `default() {}` / `default: function () {}`.
        Expression::FunctionExpression(function) => {
            if let Some(body) = function.body.as_ref() {
                visitor.visit_function_body(body);
            }
        }
        // `default: () => this.x`. An arrow has no own `this`, so a `this`
        // reference here captures the (module) enclosing scope.
        Expression::ArrowFunctionExpression(arrow) => {
            for statement in &arrow.body.statements {
                visitor.visit_statement(statement);
            }
        }
        _ => {}
    }
}

/// Reports every `this` reference bound to the visited function's own
/// `this`-scope.
///
/// Nested regular functions rebind `this`, so they are not traversed; nested
/// arrow functions capture `this` lexically and are walked normally.
struct ThisVisitor<'rule> {
    offset: usize,
    result: &'rule mut ScriptLintResult,
}

impl<'a> Visit<'a> for ThisVisitor<'_> {
    // A non-arrow nested function gets its own `this`; do not descend.
    fn visit_function(&mut self, _it: &Function<'a>, _flags: oxc_syntax::scope::ScopeFlags) {}

    fn visit_this_expression(&mut self, it: &ThisExpression) {
        self.report(it.span);
    }
}

impl ThisVisitor<'_> {
    fn report(&mut self, span: Span) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        let diagnostic = LintDiagnostic::error(
            META.name,
            "Unexpected `this` in a prop default/validator function.",
            start,
            end,
        )
        .with_label("`this` is not the component instance here", start, end)
        .with_help(
            "Vue 3 no longer binds the component instance as `this` in a prop default or \
             validator function. Use the raw props argument (`default(props) { ... }`) or move \
             the logic into the component's setup/data.",
        );
        self.result.add_diagnostic(diagnostic);
    }
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

fn find_props_object<'a>(options: &'a ObjectExpression<'a>) -> Option<&'a ObjectExpression<'a>> {
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if matches!(property_key_name(&property.key), Some("props"))
            && let Expression::ObjectExpression(object) = &property.value
        {
            return Some(object);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Component options resolution (export default / defineComponent).
//
// Mirrors the resolution in `no_dupe_keys` / `no_side_effects_in_computed`: a
// plain object, an identifier bound to one, or a `defineComponent(...)`
// wrapper, optionally through TS expression wrappers.
// ---------------------------------------------------------------------------

fn find_component_options<'a>(program: &'a Program<'a>) -> Option<&'a ObjectExpression<'a>> {
    let mut bindings: FxHashMap<&'a str, &'a ObjectExpression<'a>> = FxHashMap::default();

    for statement in program.body.iter() {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            if let BindingPattern::BindingIdentifier(id) = &declarator.id
                && let Some(object) = options_from_expression(init, &bindings)
            {
                bindings.insert(id.name.as_str(), object);
            }
        }
    }

    for statement in program.body.iter() {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        if let Some(object) = options_from_export(&export.declaration, &bindings) {
            return Some(object);
        }
    }

    None
}

fn options_from_export<'a>(
    declaration: &'a ExportDefaultDeclarationKind<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ObjectExpression(object) => Some(object),
        ExportDefaultDeclarationKind::CallExpression(call) => options_from_call(call, bindings),
        ExportDefaultDeclarationKind::Identifier(identifier) => {
            bindings.get(identifier.name.as_str()).copied()
        }
        ExportDefaultDeclarationKind::ParenthesizedExpression(paren) => {
            options_from_expression(&paren.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            options_from_expression(&ts_as.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            options_from_expression(&ts_satisfies.expression, bindings)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            options_from_expression(&ts_non_null.expression, bindings)
        }
        _ => None,
    }
}

fn options_from_expression<'a>(
    expression: &'a Expression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression {
        Expression::ObjectExpression(object) => Some(object),
        Expression::CallExpression(call) => options_from_call(call, bindings),
        Expression::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        Expression::ParenthesizedExpression(paren) => {
            options_from_expression(&paren.expression, bindings)
        }
        Expression::TSAsExpression(ts_as) => options_from_expression(&ts_as.expression, bindings),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            options_from_expression(&ts_satisfies.expression, bindings)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            options_from_expression(&ts_non_null.expression, bindings)
        }
        _ => None,
    }
}

fn options_from_call<'a>(
    call: &'a CallExpression<'a>,
    bindings: &FxHashMap<&'a str, &'a ObjectExpression<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    if !matches!(callee.name.as_str(), "defineComponent" | "_defineComponent") {
        return None;
    }
    match call.arguments.first()? {
        Argument::ObjectExpression(object) => Some(object),
        Argument::Identifier(identifier) => bindings.get(identifier.name.as_str()).copied(),
        argument => argument
            .as_expression()
            .and_then(|expression| options_from_expression(expression, bindings)),
    }
}

#[cfg(test)]
mod tests;
