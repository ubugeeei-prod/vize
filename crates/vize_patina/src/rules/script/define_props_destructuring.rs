//! script/define-props-destructuring
//!
//! Match Vue's props destructuring style preference. The default requires
//! destructuring when assigned; `always` also checks bare calls and `never`
//! preserves explicit props-object access. Vue 3.5 destructures remain reactive.
//!
//! ### Invalid
//! ```ts
//! const props = defineProps<{ foo: string }>()
//! ```
//!
//! ### Valid
//! ```ts
//! const { foo, bar = 'default' } = defineProps<{ foo: string; bar?: string }>()
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    BindingPattern, CallExpression, Expression, ExpressionStatement, Program, VariableDeclarator,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_expression_statement, walk_variable_declarator},
};
use oxc_span::{GetSpan, Span};
use vize_l0::config::PropsDestructureMode;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/define-props-destructuring",
    description: "Enforce consistent style for defineProps destructuring in <script setup>",
    default_severity: Severity::Warning,
};

/// Use the default `only-when-assigned` props destructuring preference.
pub struct DefinePropsDestructuring;

impl DefinePropsDestructuring {
    pub fn configured(mode: PropsDestructureMode) -> impl ScriptRule {
        ConfiguredDefinePropsDestructuring(mode)
    }
}

struct ConfiguredDefinePropsDestructuring(PropsDestructureMode);

impl ScriptRule for DefinePropsDestructuring {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }
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
        check(program, offset, result, PropsDestructureMode::default());
    }
}

impl ScriptRule for ConfiguredDefinePropsDestructuring {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }
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
        check(program, offset, result, self.0);
    }
}

fn check<'a>(
    program: &'a Program<'a>,
    offset: usize,
    result: &mut ScriptLintResult,
    mode: PropsDestructureMode,
) {
    DefinePropsDestructuringVisitor {
        offset,
        result,
        mode,
    }
    .visit_program(program);
}

struct DefinePropsDestructuringVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    mode: PropsDestructureMode,
}

impl DefinePropsDestructuringVisitor<'_> {
    fn report(&mut self, span: Span, message: &'static str, help: &'static str) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        self.result
            .add_diagnostic(LintDiagnostic::warn(META.name, message, start, end).with_help(help));
    }
    fn prefer_destructuring(&mut self, span: Span) {
        self.report(span, "Prefer destructuring the return value of defineProps().", "Use `const { prop = defaultValue } = defineProps<Props>()`; Vue 3.5+ preserves reactivity.");
    }
}

impl<'a> Visit<'a> for DefinePropsDestructuringVisitor<'_> {
    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        if let Some(init) = &it.init
            && let Some(with_defaults) = define_props_call(init)
        {
            let destructured = matches!(
                it.id,
                BindingPattern::ObjectPattern(_) | BindingPattern::ArrayPattern(_)
            );
            match self.mode {
                PropsDestructureMode::Never if destructured => self.report(it.id.span(), "Avoid destructuring the return value of defineProps().", "Assign props to a single binding and access `props.foo` to follow the configured style preference."),
                PropsDestructureMode::Never => {},
                _ if !destructured => self.prefer_destructuring(it.id.span()),
                _ if with_defaults => self.report(init.span(), "Avoid using withDefaults() with props destructuring.", "Put defaults directly in the destructuring pattern for Vue 3.5+."),
                _ => {},
            }
        }
        walk_variable_declarator(self, it);
    }

    fn visit_expression_statement(&mut self, it: &ExpressionStatement<'a>) {
        if self.mode == PropsDestructureMode::Always && define_props_call(&it.expression).is_some()
        {
            self.prefer_destructuring(it.expression.span());
        }
        walk_expression_statement(self, it);
    }
}

/// Recognize the compiler macro and its `withDefaults` wrapper, while skipping
/// calls that declare no props and member calls belonging to other APIs.
fn define_props_call(expression: &Expression<'_>) -> Option<bool> {
    let Expression::CallExpression(call) = expression.get_inner_expression() else {
        return None;
    };
    if call_is_named(call, "defineProps") {
        return (!call.arguments.is_empty() || call.type_arguments.is_some()).then_some(false);
    }
    if call_is_named(call, "withDefaults")
        && let Some(first) = call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
    {
        return define_props_call(first).map(|_| true);
    }
    None
}

fn call_is_named(call: &CallExpression<'_>, name: &str) -> bool {
    matches!(call.callee.get_inner_expression(), Expression::Identifier(identifier) if identifier.name.as_str() == name)
}
#[cfg(test)]
mod tests {
    use super::DefinePropsDestructuring;
    use crate::rules::script::ScriptLinter;
    use vize_l0::config::PropsDestructureMode;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(DefinePropsDestructuring::configured(
            PropsDestructureMode::Never,
        )));
        linter
    }

    #[test]
    fn test_valid_assigned_to_identifier() {
        let result = create_linter().lint("const props = defineProps<{ foo: string }>()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_no_assignment() {
        let result = create_linter().lint("defineProps<{ foo: string }>()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_unrelated_destructure() {
        let result = create_linter().lint("const { foo } = someObject", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_object_destructure() {
        let result = create_linter().lint(
            "const { foo, bar } = defineProps<{ foo: string; bar?: number }>()",
            0,
        );
        assert_eq!(result.warning_count, 1);
        assert_eq!(
            result.diagnostics[0].help.as_deref(),
            Some(
                "Assign props to a single binding and access `props.foo` to follow the configured style preference."
            )
        );
    }

    #[test]
    fn test_invalid_object_destructure_with_defaults() {
        let result =
            create_linter().lint("const { count = 0 } = defineProps<{ count?: number }>()", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_array_destructure() {
        let result = create_linter().lint("const [first] = defineProps<[string]>()", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_with_defaults_destructure() {
        let result = create_linter().lint(
            "const { count = 0 } = withDefaults(defineProps<{ count?: number }>(), { count: 0 })",
            0,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_offset_applied() {
        let source = "const { foo } = defineProps<{ foo: string }>()";
        let result = create_linter().lint(source, 200);
        assert_eq!(result.warning_count, 1);
        let pattern_start = source.find('{').unwrap() as u32 + 200;
        assert_eq!(result.diagnostics[0].start, pattern_start);
    }
}
