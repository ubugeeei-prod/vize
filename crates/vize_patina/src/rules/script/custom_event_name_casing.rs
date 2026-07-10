//! script/custom-event-name-casing
//!
//! Enforce camelCase for emitted custom event names.
//!
//! eslint-plugin-vue recommends a consistent casing for the events a component
//! emits. For Vue 3 the default is **camelCase**, so an emit whose event name is
//! kebab-case (`'my-event'`) or PascalCase (`'MyEvent'`) is inconsistent with the
//! recommended casing and is reported.
//!
//! This walks the emit call sites and checks the string-literal event name (the
//! first argument): a call to the captured `defineEmits` binding
//! (`const emit = defineEmits(...)` then `emit('my-event')`), or a member call
//! whose property is `emit`/`$emit` (`ctx.emit('my-event')`,
//! `this.$emit('my-event')`). Only string-literal event names are checked; a
//! dynamic name (`emit(eventName)`) carries no literal to inspect and is skipped.
//! The `update:` prefix used by `v-model` (`'update:modelValue'`) is permitted.
//!
//! Mirrors [`vue/custom-event-name-casing`](https://eslint.vuejs.org/rules/custom-event-name-casing.html)
//! with the Vue 3 default (`camelCase`).
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! const emit = defineEmits(['my-event'])
//! emit('my-event')         // kebab-case → report
//! ```
//!
//! ### Valid
//! ```ts
//! const emit = defineEmits(['myEvent'])
//! emit('myEvent')
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Expression, Program, Statement, StringLiteral,
};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use vize_carton::CompactString;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/custom-event-name-casing",
    description: "Enforce camelCase for emitted custom event names",
    default_severity: Severity::Error,
};

/// Enforce camelCase for emitted custom event names.
pub struct CustomEventNameCasing;

impl ScriptRule for CustomEventNameCasing {
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
        // The `<script setup>` emit function captured from `defineEmits(...)`, if
        // any. A bare `defineEmits([...])` without a binding cannot be tracked.
        let emit_binding = find_emit_binding(program);

        let mut visitor = CustomEventNameCasingVisitor {
            offset,
            result,
            emit_binding,
        };
        visitor.visit_program(program);
    }
}

struct CustomEventNameCasingVisitor<'a, 'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    emit_binding: Option<&'a str>,
}

impl<'a> Visit<'a> for CustomEventNameCasingVisitor<'a, '_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if self.is_emit_call(it)
            && let Some(Argument::StringLiteral(literal)) = it.arguments.first()
        {
            self.check_event_name(literal);
        }
        walk_call_expression(self, it);
    }
}

impl<'a> CustomEventNameCasingVisitor<'a, '_> {
    /// Whether the call is an emit of a custom event: a call to the captured
    /// `defineEmits` binding (`emit(...)`) or a member call whose property is
    /// `emit`/`$emit` (`ctx.emit(...)`, `this.$emit(...)`).
    fn is_emit_call(&self, call: &CallExpression<'a>) -> bool {
        match &call.callee {
            Expression::Identifier(identifier) => {
                self.emit_binding == Some(identifier.name.as_str())
            }
            Expression::StaticMemberExpression(member) => {
                matches!(member.property.name.as_str(), "emit" | "$emit")
            }
            _ => false,
        }
    }

    fn check_event_name(&mut self, literal: &StringLiteral<'_>) {
        let value = literal.value.as_str();
        if is_camel_case_event(value) {
            return;
        }

        let start = self.offset as u32 + literal.span.start;
        let end = self.offset as u32 + literal.span.end;

        let mut message = CompactString::with_capacity(value.len() + 40);
        message.push_str("Custom event name '");
        message.push_str(value);
        message.push_str("' is not camelCase.");

        let diagnostic = LintDiagnostic::error(META.name, message, start, end)
            .with_label("expected camelCase", start, end)
            .with_help(
                "Vue 3 recommends camelCase for emitted event names; rename this event \
                 to camelCase (e.g. `myEvent`).",
            );
        self.result.add_diagnostic(diagnostic);
    }
}

/// Whether `value` is an acceptable camelCase event name. Each `:`-separated
/// segment (so the `v-model` `update:modelValue` form is allowed) must match
/// `^[a-z][a-zA-Z0-9]*$`: a lowercase first character followed by alphanumerics.
fn is_camel_case_event(value: &str) -> bool {
    !value.is_empty() && value.split(':').all(is_camel_case_segment)
}

/// Whether a single segment matches `^[a-z][a-zA-Z0-9]*$`.
fn is_camel_case_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    match chars.next() {
        Some(first) if first.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric())
}

/// The identifier a top-level `const <id> = defineEmits(...)` binds the emit
/// function to, if present. An unassigned `defineEmits(...)` returns `None`.
fn find_emit_binding<'a>(program: &'a Program<'a>) -> Option<&'a str> {
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let BindingPattern::BindingIdentifier(id) = &declarator.id else {
                continue;
            };
            if declarator
                .init
                .as_ref()
                .is_some_and(|init| is_define_emits_call(init))
            {
                return Some(id.name.as_str());
            }
        }
    }
    None
}

/// Whether the expression is a `defineEmits(...)` call, unwrapping the TS
/// `as`/`satisfies`/non-null and parenthesized wrappers.
fn is_define_emits_call(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::CallExpression(call) => matches!(
            &call.callee,
            Expression::Identifier(identifier) if identifier.name.as_str() == "defineEmits"
        ),
        Expression::ParenthesizedExpression(paren) => is_define_emits_call(&paren.expression),
        Expression::TSAsExpression(ts) => is_define_emits_call(&ts.expression),
        Expression::TSSatisfiesExpression(ts) => is_define_emits_call(&ts.expression),
        Expression::TSNonNullExpression(ts) => is_define_emits_call(&ts.expression),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::CustomEventNameCasing;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(CustomEventNameCasing));
        linter
    }

    #[test]
    fn test_valid_camel_case_setup_emit() {
        let source = r#"
const emit = defineEmits(['myEvent'])
emit('myEvent')
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_single_word_event() {
        let source = r#"
const emit = defineEmits(['change'])
emit('change')
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_update_model_value() {
        // The `update:` prefix used by `v-model` is permitted.
        let result = create_linter().lint("this.$emit('update:modelValue', value)", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_kebab_case_setup_emit() {
        let source = r#"
const emit = defineEmits(['my-event'])
emit('my-event')
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_pascal_case_dollar_emit() {
        let result = create_linter().lint("this.$emit('MyEvent')", 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_kebab_case_dollar_emit() {
        let result = create_linter().lint("this.$emit('my-event')", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_context_emit_kebab() {
        // A setup-context member call (`ctx.emit('...')`) is checked too.
        let result = create_linter().lint("ctx.emit('my-event')", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_context_emit_camel() {
        let result = create_linter().lint("ctx.emit('myEvent')", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_dynamic_event_name_not_checked() {
        // A non-string-literal event name carries no literal to inspect.
        let source = r#"
const emit = defineEmits(['myEvent'])
const name = 'my-event'
emit(name)
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_unassigned_define_emits_call_not_tracked() {
        // Without a binding the `emit(...)` identifier cannot be resolved, so the
        // bare `emit` identifier call is not treated as an emit.
        let source = r#"
defineEmits(['my-event'])
emit('my-event')
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_custom_emit_binding_name() {
        let source = r#"
const myEmit = defineEmits(['change'])
myEmit('my-event')
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_options_api_this_emit_in_method() {
        let source = r#"
export default {
  methods: {
    submit() {
      this.$emit('my-event')
    }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_plain_identifier_call_not_emit() {
        // A call to some other function is not an emit, even with a kebab string.
        let result = create_linter().lint("notify('my-event')", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_multiple_invalid_events() {
        let source = r#"
const emit = defineEmits(['my-event', 'OtherEvent'])
emit('my-event')
emit('OtherEvent')
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn test_offset_applied() {
        let result = create_linter().lint("this.$emit('my-event')", 30);
        assert_eq!(result.error_count, 1);
        assert_eq!(result.diagnostics[0].start, 30 + 11);
    }
}
