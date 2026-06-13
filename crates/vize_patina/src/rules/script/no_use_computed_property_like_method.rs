//! script/no-use-computed-property-like-method
//!
//! Disallow calling an Options API `computed` property like a method. A computed
//! entry exposes a *value*, not a function, so `this.fullName()` (when
//! `fullName` is a computed) evaluates the value and immediately calls it —
//! throwing at runtime unless it happens to return a function. The call
//! parentheses are almost always a mistake for `this.fullName`.
//!
//! Port of [`vue/no-use-computed-property-like-method`](https://eslint.vuejs.org/rules/no-use-computed-property-like-method.html),
//! scoped conservatively to the Options API: only `this.<computedName>(...)`
//! member calls are flagged, where `<computedName>` is declared in `computed`.

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, CallExpression, ExportDefaultDeclarationKind, Expression, ObjectExpression,
    ObjectPropertyKind, Program, PropertyKey, Statement,
};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::Span;
use vize_carton::{CompactString, FxHashMap, FxHashSet};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-use-computed-property-like-method",
    description: "Disallow calling an Options API computed property like a method",
    default_severity: Severity::Error,
};

/// Disallow calling an Options API `computed` property like a method.
pub struct NoUseComputedPropertyLikeMethod;

impl ScriptRule for NoUseComputedPropertyLikeMethod {
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
        let computed_names = collect_computed_names(options);
        if computed_names.is_empty() {
            return;
        }
        let mut visitor = ComputedCallVisitor {
            computed_names: &computed_names,
            offset,
            result,
            fn_depth: 0,
        };
        visitor.visit_object_expression(options);
    }
}

/// Collect the names declared in the `computed` option. Only plain
/// (non-computed-key) properties contribute; spreads like `...mapGetters([..])`
/// are skipped since their member names are not statically known.
fn collect_computed_names<'a>(options: &'a ObjectExpression<'a>) -> FxHashSet<CompactString> {
    let mut names = FxHashSet::default();
    let Some(computed) = find_computed_object(options) else {
        return names;
    };
    for property in &computed.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if let Some(name) = property_key_name(&property.key) {
            names.insert(CompactString::from(name));
        }
    }
    names
}

/// Walks the component and reports `this.<computedName>(...)` member calls.
///
/// A direct member function binds `this` to the component instance, so a call
/// there is reported (`fn_depth == 1`). A non-arrow function nested inside a
/// member rebinds `this`, so deeper calls are skipped to avoid false positives.
/// Arrow functions keep the lexical `this` and do not change the depth.
struct ComputedCallVisitor<'rule> {
    computed_names: &'rule FxHashSet<CompactString>,
    offset: usize,
    result: &'rule mut ScriptLintResult,
    /// Non-arrow function nesting depth from the options object; `1` is a direct
    /// member (its `this` is the instance), deeper layers have rebound `this`.
    fn_depth: u32,
}

impl<'a> Visit<'a> for ComputedCallVisitor<'_> {
    fn visit_function(
        &mut self,
        it: &oxc_ast::ast::Function<'a>,
        flags: oxc_syntax::scope::ScopeFlags,
    ) {
        self.fn_depth += 1;
        oxc_ast_visit::walk::walk_function(self, it, flags);
        self.fn_depth -= 1;
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if self.fn_depth == 1
            && let Expression::StaticMemberExpression(member) = &it.callee
            && matches!(&member.object, Expression::ThisExpression(_))
            && self.computed_names.contains(member.property.name.as_str())
        {
            self.report(it.span, member.property.name.as_str());
        }
        walk_call_expression(self, it);
    }
}

impl ComputedCallVisitor<'_> {
    fn report(&mut self, span: Span, name: &str) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        let mut message = CompactString::with_capacity(name.len() + 56);
        message.push_str("'");
        message.push_str(name);
        message.push_str("' is a computed property and must not be called like a method.");
        let diagnostic = LintDiagnostic::error(META.name, message, start, end)
            .with_label("computed value called as a function", start, end)
            .with_help(
                "A computed property exposes a value, not a function. Read it as \
                 `this.<name>` (drop the call parentheses), or move the logic into a \
                 `method` if you need to invoke it.",
            );
        self.result.add_diagnostic(diagnostic);
    }
}

fn find_computed_object<'a>(options: &'a ObjectExpression<'a>) -> Option<&'a ObjectExpression<'a>> {
    for property in &options.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        if matches!(property_key_name(&property.key), Some("computed"))
            && let Expression::ObjectExpression(object) = &property.value
        {
            return Some(object);
        }
    }
    None
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

// Component options resolution (export default / defineComponent), mirroring
// `no_side_effects_in_computed`: a plain object, an identifier bound to one, or
// a `defineComponent(...)` wrapper, optionally through TS expression wrappers.
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
            if let oxc_ast::ast::BindingPattern::BindingIdentifier(id) = &declarator.id
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
    // Every export-default kind we care about is also an `Expression` variant,
    // so route through the shared expression resolver.
    options_from_expression(declaration.as_expression()?, bindings)
}

/// Resolve an expression to the component options object, peeling
/// parenthesized/TS-cast wrappers, following identifier bindings, and entering
/// `defineComponent(...)` calls.
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
        Expression::TSSatisfiesExpression(ts) => options_from_expression(&ts.expression, bindings),
        Expression::TSNonNullExpression(ts) => options_from_expression(&ts.expression, bindings),
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
