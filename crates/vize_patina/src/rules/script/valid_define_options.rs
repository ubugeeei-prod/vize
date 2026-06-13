//! script/valid-define-options
//!
//! Enforce valid usage of the `defineOptions` compiler macro in `<script setup>`.
//!
//! `defineOptions` declares component options that have no dedicated macro (for
//! example `name` or `inheritAttrs`). It has three constraints, each enforced
//! here:
//!
//! 1. It may be called **at most once** per `<script setup>` block.
//! 2. It takes a **single object-literal** argument.
//! 3. That object must **not** declare `props`, `emits`, `expose`, or `slots`:
//!    those have dedicated macros (`defineProps`, `defineEmits`, `defineExpose`,
//!    `defineSlots`) and are rejected by the compiler inside `defineOptions`.
//!
//! Mirrors [`vue/valid-define-options`](https://eslint.vuejs.org/rules/valid-define-options.html).
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! defineOptions({ props: ['foo'] })   // use defineProps instead
//! defineOptions({ name: 'Foo' })
//! defineOptions({ name: 'Bar' })      // duplicate call
//! defineOptions('Foo')                // not an object literal
//! ```
//!
//! ### Valid
//! ```ts
//! defineOptions({ name: 'Foo', inheritAttrs: false })
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, CallExpression, Expression, ObjectPropertyKind, Program, PropertyKey,
};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::{GetSpan, Span};
use vize_carton::CompactString;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/valid-define-options",
    description: "Enforce valid defineOptions() usage (single object arg, no props/emits/expose/slots)",
    default_severity: Severity::Error,
};

/// Options that have dedicated macros and must not appear inside
/// `defineOptions({ ... })`.
const FORBIDDEN_KEYS: [&str; 4] = ["props", "emits", "expose", "slots"];

/// Enforce valid `defineOptions` usage.
pub struct ValidDefineOptions;

impl ScriptRule for ValidDefineOptions {
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
        let mut visitor = ValidDefineOptionsVisitor {
            offset,
            result,
            seen_call: false,
        };
        visitor.visit_program(program);
    }
}

struct ValidDefineOptionsVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    seen_call: bool,
}

impl<'a> Visit<'a> for ValidDefineOptionsVisitor<'_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if is_define_options_call(it) {
            self.check_call(it);
        }
        walk_call_expression(self, it);
    }
}

impl ValidDefineOptionsVisitor<'_> {
    fn check_call(&mut self, call: &CallExpression<'_>) {
        // (a) at most one call.
        if self.seen_call {
            self.report(
                call.span,
                "defineOptions() has already been called.",
                "Merge all component options into a single defineOptions() call; \
                 only one is allowed per <script setup> block.",
            );
            return;
        }
        self.seen_call = true;

        // (b) a single object-literal argument.
        let Some(Argument::ObjectExpression(object)) = call.arguments.first() else {
            self.report(
                call.span,
                "defineOptions() expects a single object-literal argument.",
                "Pass an object literal, e.g. `defineOptions({ name: 'Foo', inheritAttrs: false })`.",
            );
            return;
        };

        // (c) no props / emits / expose / slots keys.
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            if property.computed {
                continue;
            }
            let Some(name) = property_key_name(&property.key) else {
                continue;
            };
            if let Some(forbidden) = FORBIDDEN_KEYS.iter().find(|key| **key == name) {
                self.report_forbidden(property.key.span(), forbidden);
            }
        }
    }

    fn report(&mut self, span: Span, message: &'static str, help: &'static str) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        self.result
            .add_diagnostic(LintDiagnostic::error(META.name, message, start, end).with_help(help));
    }

    fn report_forbidden(&mut self, span: Span, key: &str) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;

        let mut message = CompactString::with_capacity(key.len() + 48);
        message.push_str("'");
        message.push_str(key);
        message.push_str("' is not allowed inside defineOptions().");

        let mut help = CompactString::with_capacity(key.len() + 32);
        help.push_str("Use the dedicated `define");
        help.push_str(macro_for_key(key));
        help.push_str("()` macro instead of declaring it in defineOptions().");

        self.result.add_diagnostic(
            LintDiagnostic::error(META.name, message, start, end)
                .with_label("declared in defineOptions()", start, end)
                .with_help(help),
        );
    }
}

/// The dedicated macro suffix for a forbidden key (`props` -> `Props`).
fn macro_for_key(key: &str) -> &'static str {
    match key {
        "props" => "Props",
        "emits" => "Emits",
        "expose" => "Expose",
        "slots" => "Slots",
        _ => "",
    }
}

/// Whether the callee is the bare `defineOptions` compiler macro.
fn is_define_options_call(call: &CallExpression<'_>) -> bool {
    matches!(
        &call.callee,
        Expression::Identifier(identifier) if identifier.name.as_str() == "defineOptions"
    )
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
    use super::ValidDefineOptions;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(ValidDefineOptions));
        linter
    }

    #[test]
    fn test_valid_name_and_inherit_attrs() {
        let result = create_linter().lint("defineOptions({ name: 'Foo', inheritAttrs: false })", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_empty_object() {
        let result = create_linter().lint("defineOptions({})", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_no_define_options() {
        let result = create_linter().lint("const x = defineProps<{ a: number }>()", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_duplicate_call() {
        let result = create_linter().lint(
            "defineOptions({ name: 'Foo' })\ndefineOptions({ inheritAttrs: false })",
            0,
        );
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_non_object_argument() {
        let result = create_linter().lint("defineOptions('Foo')", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_no_argument() {
        let result = create_linter().lint("defineOptions()", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_props_key() {
        let result = create_linter().lint("defineOptions({ props: ['foo'] })", 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_emits_key() {
        let result = create_linter().lint("defineOptions({ emits: ['change'] })", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_expose_and_slots_keys() {
        let result = create_linter().lint("defineOptions({ expose: ['a'], slots: {} })", 0);
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn test_invalid_forbidden_key_string_literal() {
        let result = create_linter().lint("defineOptions({ 'props': ['foo'] })", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_name_alongside_forbidden_key_reports_only_forbidden() {
        let result = create_linter().lint("defineOptions({ name: 'Foo', props: ['bar'] })", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_offset_applied() {
        let result = create_linter().lint("defineOptions()", 30);
        assert_eq!(result.error_count, 1);
        assert_eq!(result.diagnostics[0].start, 30);
    }
}
