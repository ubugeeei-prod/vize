//! script/valid-define-props
//!
//! Enforce valid usage of the `defineProps` compiler macro in `<script setup>`.
//!
//! `defineProps` declares a component's props. Its constraints, each enforced
//! here:
//!
//! 1. It may be called **at most once** per `<script setup>` block; every call
//!    is reported otherwise.
//! 2. It must **not** be given both a type argument and a runtime argument
//!    (`defineProps<{ ... }>({ ... })`): props are declared either by type *or*
//!    by runtime object, never both.
//! 3. Its runtime argument must **not** reference locally-declared variables:
//!    `defineProps` is hoisted by the compiler, so referencing module-local
//!    bindings (an `enum` included) is unsafe. References are resolved through
//!    scope: a type position and a parameter of a function inside the argument
//!    are not local references.
//! 4. A call with neither declares no prop, unless the plain `<script>`'s
//!    `export default {}` has a `props` option; a call next to such an option
//!    declares the props twice.
//!
//! Mirrors [`vue/valid-define-props`](https://eslint.vuejs.org/rules/valid-define-props.html).
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! defineProps<{ foo: string }>({ foo: String }) // both type and runtime args
//! const foo = 'bar'
//! defineProps({ foo })                          // references local variable
//! defineProps({ foo: String })
//! defineProps({ bar: Number })                  // duplicate call
//! ```
//!
//! ### Valid
//! ```ts
//! defineProps<{ foo: string }>()
//! defineProps({ foo: String })
//! ```

use super::macro_validity::{Finding, check};
use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::Program;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/valid-define-props",
    description: "Enforce valid defineProps() usage (single call, not both type and runtime args, no local references)",
    default_severity: Severity::Error,
};

/// Enforce valid `defineProps` usage.
pub struct ValidDefineProps;

impl ScriptRule for ValidDefineProps {
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
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        self.check_program_with_sfc(program, source, offset, SfcScriptContext::default(), result);
    }

    fn check_program_with_sfc<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        if sfc.is_sfc && !sfc.is_script_setup {
            return;
        }
        for finding in check(program, sfc, "defineProps", "props") {
            let (span, message, help) = match finding {
                Finding::Multiple(span) => (
                    span,
                    "defineProps() has been called multiple times.",
                    "Merge all props into a single defineProps() call; only one is \
                     allowed per <script setup> block.",
                ),
                Finding::HasTypeAndArg(span) => (
                    span,
                    "defineProps() has both a type argument and a runtime argument.",
                    "Declare props with either a type argument (`defineProps<{ ... }>()`) \
                     or a runtime argument (`defineProps({ ... })`), not both.",
                ),
                Finding::ReferencingLocally(span) => (
                    span,
                    "defineProps() is referencing locally declared variables.",
                    "Inline the value or move the declaration out of <script setup>; \
                     defineProps() is hoisted and cannot reference module-local bindings.",
                ),
                Finding::NotDefined(span) => (
                    span,
                    "defineProps() does not define any props.",
                    "Declare the props with a type argument or a runtime argument.",
                ),
                Finding::DefinedInBoth(span) => (
                    span,
                    "Props are defined in both defineProps() and export default {}.",
                    "Declare the props in one place.",
                ),
            };
            let start = offset as u32 + span.start;
            let end = offset as u32 + span.end;
            result.add_diagnostic(
                LintDiagnostic::error(META.name, message, start, end).with_help(help),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ValidDefineProps;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(ValidDefineProps));
        linter
    }

    #[test]
    fn test_valid_type_only() {
        let result = create_linter().lint("defineProps<{ foo: string }>()", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_runtime_only() {
        let result = create_linter().lint("defineProps({ foo: String })", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_runtime_array() {
        let result = create_linter().lint("defineProps(['foo', 'bar'])", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_empty_runtime() {
        let result = create_linter().lint("defineProps({})", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_no_define_props() {
        let result = create_linter().lint("const x = defineEmits<{ a: [] }>()", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_references_imported_type_constructor() {
        // `String`/`Number` are globals, not locally declared.
        let result = create_linter().lint("defineProps({ foo: String, bar: Number })", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_assigned_to_binding() {
        let result = create_linter().lint("const props = defineProps<{ foo: string }>()", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_both_type_and_runtime() {
        let result = create_linter().lint("defineProps<{ foo: string }>({ foo: String })", 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_references_local_variable() {
        let result = create_linter().lint("const foo = { type: String }\ndefineProps({ foo })", 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_references_local_variable_as_value() {
        let result = create_linter().lint("const def = String\ndefineProps({ foo: def })", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_references_local_function() {
        let result = create_linter().lint(
            "function validate() {}\ndefineProps({ foo: { validator: validate } })",
            0,
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_duplicate_call() {
        let result = create_linter().lint(
            "defineProps({ foo: String })\ndefineProps({ bar: Number })",
            0,
        );
        assert_eq!(result.error_count, 2);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_duplicate_call_type_args() {
        let result = create_linter().lint(
            "defineProps<{ foo: string }>()\ndefineProps<{ bar: number }>()",
            0,
        );
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn test_both_type_and_runtime_takes_priority_over_local_ref() {
        // The both-args error short-circuits; only one diagnostic is produced.
        let result = create_linter().lint(
            "const foo = String\ndefineProps<{ foo: string }>({ foo })",
            0,
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_offset_applied() {
        let result = create_linter().lint("defineProps<{ foo: string }>({ foo: String })", 30);
        assert_eq!(result.error_count, 1);
        assert!(result.diagnostics[0].start >= 30);
    }
}
