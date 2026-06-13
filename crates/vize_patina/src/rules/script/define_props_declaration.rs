//! script/define-props-declaration
//!
//! Enforce the type-based `defineProps<{ ... }>()` declaration over the
//! runtime/object form `defineProps({ ... })` in `<script setup>`.
//!
//! The type-based form keeps prop typing and the runtime declaration in a
//! single source of truth, gives better editor support, and avoids the
//! redundant `PropType<T>` casts the runtime form needs. This mirrors
//! [`vue/define-props-declaration`](https://eslint.vuejs.org/rules/define-props-declaration.html)
//! from eslint-plugin-vue, defaulting to the `type-based` preference.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! // Runtime / object form
//! const props = defineProps({
//!   kind: { type: String },
//!   count: { type: Number, default: 0 },
//! })
//!
//! // Runtime / array form
//! const props = defineProps(['kind', 'count'])
//! ```
//!
//! ### Valid
//! ```ts
//! // Type-based form
//! const props = defineProps<{ kind: string; count?: number }>()
//!
//! // Type-based with a referenced interface
//! const props = defineProps<Props>()
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{CallExpression, Expression, Program};
use oxc_ast_visit::{Visit, walk::walk_call_expression};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/define-props-declaration",
    description: "Enforce type-based defineProps<{ ... }>() over the runtime/object form",
    default_severity: Severity::Warning,
};

const MESSAGE: &str =
    "Prefer the type-based `defineProps<{ ... }>()` declaration over the runtime form";
const HELP: &str = "Move the props to a type argument, e.g. `defineProps<{ count?: number }>()`, \
     instead of passing a runtime object/array.";

/// Enforce the type-based defineProps declaration.
pub struct DefinePropsDeclaration;

impl ScriptRule for DefinePropsDeclaration {
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
        let mut visitor = DefinePropsDeclarationVisitor { offset, result };
        visitor.visit_program(program);
    }
}

struct DefinePropsDeclarationVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for DefinePropsDeclarationVisitor<'_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if is_define_props_call(it) && has_runtime_argument(it) {
            let start = self.offset as u32 + it.span.start;
            let end = self.offset as u32 + it.span.end;
            self.result.add_diagnostic(
                LintDiagnostic::warn(META.name, MESSAGE, start, end).with_help(HELP),
            );
        }

        walk_call_expression(self, it);
    }
}

/// Whether the callee is the bare `defineProps` compiler macro.
fn is_define_props_call(call: &CallExpression<'_>) -> bool {
    matches!(
        &call.callee,
        Expression::Identifier(identifier) if identifier.name.as_str() == "defineProps"
    )
}

/// Whether the call carries a runtime argument (an object/array literal) rather
/// than relying solely on a type argument.
///
/// `defineProps<Props>()` and `defineProps<{ a: number }>()` carry only a type
/// argument and are valid. `defineProps({ ... })` / `defineProps([ ... ])` pass
/// a runtime value and should switch to the type-based form. A bare
/// `defineProps()` has nothing to convert and is left alone.
fn has_runtime_argument(call: &CallExpression<'_>) -> bool {
    call.arguments
        .iter()
        .any(|argument| argument.as_expression().is_some())
}

#[cfg(test)]
mod tests {
    use super::DefinePropsDeclaration;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(DefinePropsDeclaration));
        linter
    }

    #[test]
    fn test_valid_type_based_inline() {
        let linter = create_linter();
        let result = linter.lint(
            "const props = defineProps<{ kind: string; count?: number }>()",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_type_based_reference() {
        let linter = create_linter();
        let result = linter.lint("const props = defineProps<Props>()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_bare_define_props() {
        // No runtime argument and no type argument: nothing to convert.
        let linter = create_linter();
        let result = linter.lint("const props = defineProps()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_destructured_type_based() {
        let linter = create_linter();
        let result = linter.lint("const { count = 0 } = defineProps<{ count?: number }>()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_unrelated_call() {
        let linter = create_linter();
        let result = linter.lint("const e = defineEmits(['change'])", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_runtime_object() {
        let linter = create_linter();
        let result = linter.lint(
            "const props = defineProps({ kind: { type: String }, count: { type: Number, default: 0 } })",
            0,
        );
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_runtime_array() {
        let linter = create_linter();
        let result = linter.lint("const props = defineProps(['kind', 'count'])", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_runtime_without_assignment() {
        let linter = create_linter();
        let result = linter.lint("defineProps({ msg: String })", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_offset_is_applied() {
        let linter = create_linter();
        let source = "const props = defineProps({ msg: String })";
        let result = linter.lint(source, 100);
        assert_eq!(result.warning_count, 1);
        let call_start = source.find("defineProps").unwrap() as u32 + 100;
        assert_eq!(result.diagnostics[0].start, call_start);
    }
}
