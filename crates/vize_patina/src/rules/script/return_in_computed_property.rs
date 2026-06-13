//! script/return-in-computed-property
//!
//! Require a return value in every computed getter. A getter that can finish
//! without returning a value yields `undefined`, which is almost always a bug (a
//! forgotten branch, a dropped `return`, or a body run only for side effects).
//! This rule flags any computed getter whose block body has no value-returning
//! `return`.
//!
//! Both API styles are covered: the Options API (`computed: { foo() {} }` and
//! the accessor form `computed: { foo: { get() {} } }`) and the Composition API
//! (`computed(() => {})` and the writable `computed({ get() {} })` form). A
//! concise-body arrow (`() => expr`) always returns its expression and is never
//! flagged. A `return` inside a nested function is that function's own and does
//! not count toward the getter's return.
//!
//! Port of [`vue/return-in-computed-property`](https://eslint.vuejs.org/rules/return-in-computed-property.html).

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, ArrowFunctionExpression, CallExpression, Expression, Function, ObjectExpression,
    ObjectProperty, ObjectPropertyKind, Program, PropertyKey, PropertyKind, ReturnStatement,
    Statement,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_object_property},
};
use oxc_span::Span;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/return-in-computed-property",
    description: "Require a return value in every computed getter",
    default_severity: Severity::Error,
};

/// Require a return value in every computed getter.
pub struct ReturnInComputedProperty;

impl ScriptRule for ReturnInComputedProperty {
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
        // A single walk handles both API styles: `computed(...)` calls
        // (Composition API) and `computed: { ... }` option objects (Options
        // API), wherever they appear in the program.
        ComputedVisitor { offset, result }.visit_program(program);
    }
}

/// Walks the program for every computed getter and checks it returns a value.
struct ComputedVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for ComputedVisitor<'_> {
    /// Composition API: `computed(getter)` / `computed({ get() {} })`.
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if matches!(&it.callee, Expression::Identifier(id) if id.name == "computed")
            && let Some(expression) = it.arguments.first().and_then(Argument::as_expression)
            && let Some(getter) = getter_from_value(expression)
        {
            check_getter(getter, self.offset, self.result);
        }
        walk_call_expression(self, it);
    }

    /// Options API: each entry of a `computed: { ... }` option object. The walk
    /// reaches this object regardless of how the component is declared
    /// (`export default {}`, `defineComponent({})`, an identifier-bound object).
    fn visit_object_property(&mut self, it: &ObjectProperty<'a>) {
        if !it.computed
            && matches!(property_key_name(&it.key), Some("computed"))
            && let Expression::ObjectExpression(object) = &it.value
        {
            for property in &object.properties {
                if let ObjectPropertyKind::ObjectProperty(entry) = property
                    && !entry.computed
                    && let Some(getter) = getter_from_value(&entry.value)
                {
                    check_getter(getter, self.offset, self.result);
                }
            }
        }
        walk_object_property(self, it);
    }
}

/// A computed getter whose block body must contain a value-returning `return`.
enum Getter<'a> {
    /// A `function` / method getter (`foo() {}`, `get() {}`).
    Function(&'a Function<'a>),
    /// An arrow getter (`() => {}`).
    Arrow(&'a ArrowFunctionExpression<'a>),
}

/// Interpret a getter-position value: a `function`/arrow getter directly, or the
/// `get` accessor of a `{ get() {}, set() {} }` writable form.
fn getter_from_value<'a>(value: &'a Expression<'a>) -> Option<Getter<'a>> {
    match unwrap_expression(value) {
        Expression::FunctionExpression(function) => Some(Getter::Function(function)),
        Expression::ArrowFunctionExpression(arrow) => Some(Getter::Arrow(arrow)),
        Expression::ObjectExpression(object) => accessor_get(object),
        _ => None,
    }
}

/// Locate the `get` accessor inside `{ get() {}, set() {} }`.
fn accessor_get<'a>(object: &'a ObjectExpression<'a>) -> Option<Getter<'a>> {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed {
            continue;
        }
        let is_get = property.kind == PropertyKind::Get
            || matches!(property_key_name(&property.key), Some("get"));
        if !is_get {
            continue;
        }
        match unwrap_expression(&property.value) {
            Expression::FunctionExpression(function) => return Some(Getter::Function(function)),
            Expression::ArrowFunctionExpression(arrow) => return Some(Getter::Arrow(arrow)),
            _ => {}
        }
    }
    None
}

/// Report the getter unless its block body has a value-returning `return`.
fn check_getter(getter: Getter<'_>, offset: usize, result: &mut ScriptLintResult) {
    let (span, statements): (Span, _) = match getter {
        // No body (e.g. a TS overload signature): nothing to analyze.
        Getter::Function(function) => match function.body.as_ref() {
            Some(body) => (function.span, &body.statements),
            None => return,
        },
        // An expression-bodied arrow (`() => expr`) implicitly returns its
        // expression; only block bodies can be missing a return.
        Getter::Arrow(arrow) if arrow.expression => return,
        Getter::Arrow(arrow) => (arrow.span, &arrow.body.statements),
    };

    if body_returns_value(statements) {
        return;
    }

    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    result.add_diagnostic(
        LintDiagnostic::error(
            META.name,
            "Expected to return a value in this computed property.",
            start,
            end,
        )
        .with_label("computed getter without a return value", start, end)
        .with_help(
            "A computed getter must return the derived value. Add a `return` that yields a \
             value on every path through the getter.",
        ),
    );
}

/// Whether the statement list contains a `return <expr>` reachable as the
/// getter's own return (i.e. not inside a nested function scope).
fn body_returns_value(statements: &[Statement<'_>]) -> bool {
    let mut finder = ReturnFinder { found: false };
    for statement in statements {
        finder.visit_statement(statement);
        if finder.found {
            return true;
        }
    }
    false
}

/// Walks a getter body looking for a value-returning `return`. Nested function
/// and arrow scopes are not traversed: their `return` is their own.
struct ReturnFinder {
    found: bool,
}

impl<'a> Visit<'a> for ReturnFinder {
    fn visit_function(&mut self, _it: &Function<'a>, _flags: oxc_syntax::scope::ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _it: &ArrowFunctionExpression<'a>) {}

    fn visit_return_statement(&mut self, it: &ReturnStatement<'a>) {
        // `return;` produces no value; only `return <expr>` counts.
        if it.argument.is_some() {
            self.found = true;
        }
    }
}

/// Strip parentheses and TS-only wrappers so the underlying node is seen.
fn unwrap_expression<'a, 'b>(expression: &'b Expression<'a>) -> &'b Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(p) => unwrap_expression(&p.expression),
        Expression::TSAsExpression(ts) => unwrap_expression(&ts.expression),
        Expression::TSSatisfiesExpression(ts) => unwrap_expression(&ts.expression),
        Expression::TSNonNullExpression(ts) => unwrap_expression(&ts.expression),
        _ => expression,
    }
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::ReturnInComputedProperty;
    use crate::rules::script::{ScriptLintResult, ScriptLinter};

    fn lint(source: &str) -> ScriptLintResult {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(ReturnInComputedProperty));
        linter.lint(source, 0)
    }

    // --- Composition API ---

    #[test]
    fn test_valid_composition_returns() {
        // Concise arrow, block arrow with return, function expression with return.
        assert_eq!(lint("const d = computed(() => count.value * 2)").error_count, 0);
        assert_eq!(lint("const d = computed(() => { return count.value })").error_count, 0);
        assert_eq!(lint("const d = computed(function () { return count.value })").error_count, 0);
    }

    #[test]
    fn test_invalid_block_arrow_without_return() {
        let result = lint("const d = computed(() => { const v = count.value })");
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_function_expression_without_return() {
        assert_eq!(lint("const d = computed(function () { const v = count.value })").error_count, 1);
    }

    #[test]
    fn test_writable_getter() {
        // Only the getter is checked; the setter never needs to return.
        let invalid = "const v = computed({ get() { const n = count.value }, set(x) { count.value = x } })";
        assert_eq!(lint(invalid).error_count, 1);
        let valid = "const v = computed({ get() { return count.value }, set(x) { count.value = x } })";
        assert_eq!(lint(valid).error_count, 0);
    }

    #[test]
    fn test_invalid_return_only_in_nested_function() {
        // The only `return` belongs to a nested function, not the getter.
        assert_eq!(lint("const d = computed(() => { const h = () => { return count.value } })").error_count, 1);
    }

    #[test]
    fn test_valid_return_in_branch() {
        assert_eq!(lint("const s = computed(() => { if (c.value) { return 'p' } return 'n' })").error_count, 0);
    }

    #[test]
    fn test_invalid_bare_return_without_value() {
        // `return;` does not produce a value.
        assert_eq!(lint("const d = computed(() => { if (!c.value) { return } })").error_count, 1);
    }

    #[test]
    fn test_ignores_non_computed_call() {
        assert_eq!(lint("const r = useFoo(() => { const x = 1 })").error_count, 0);
    }

    #[test]
    fn test_multiple_invalid_computed_reported() {
        assert_eq!(lint("const a = computed(() => { let x = 1 })\nconst b = computed(() => { let y = 2 })").error_count, 2);
    }

    // --- Options API ---

    #[test]
    fn test_valid_options_getters() {
        // Method getter with return and concise-arrow getter.
        assert_eq!(lint("export default { computed: { total() { return this.a } } }").error_count, 0);
        assert_eq!(lint("export default { computed: { total: () => this.a } }").error_count, 0);
    }

    #[test]
    fn test_invalid_options_getter_without_return() {
        let result = lint("export default { computed: { total() { const s = this.a } } }");
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_options_accessor_getter_without_return() {
        let source = "export default { computed: { value: { get() { const n = this.count }, set(v) { this.count = v } } } }";
        assert_eq!(lint(source).error_count, 1);
    }

    #[test]
    fn test_define_component_options_getter() {
        let source = "import { defineComponent } from 'vue'\nexport default defineComponent({ computed: { total() { const s = this.a } } })";
        assert_eq!(lint(source).error_count, 1);
    }

    #[test]
    fn test_identifier_export_options_getter() {
        let source = "const c = { computed: { total() { const s = this.a } } }\nexport default c";
        assert_eq!(lint(source).error_count, 1);
    }

    #[test]
    fn test_valid_no_getter_to_check() {
        // No `computed` option, and a spread (`...mapGetters`) is ignored.
        assert_eq!(lint("export default { data() { return { count: 0 } } }").error_count, 0);
        assert_eq!(lint("export default { computed: { ...mapGetters(['count']) } }").error_count, 0);
    }

    #[test]
    fn test_multiple_options_getters_mixed() {
        let source = "export default { computed: { good() { return this.a }, bad() { const x = this.b } } }";
        assert_eq!(lint(source).error_count, 1);
    }
}
