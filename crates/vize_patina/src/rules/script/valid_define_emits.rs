//! script/valid-define-emits
//!
//! Enforce valid usage of the `defineEmits` compiler macro in `<script setup>`.
//!
//! `defineEmits` declares the events a component can emit, either via a runtime
//! argument (`defineEmits(['change'])`) or a type argument
//! (`defineEmits<{ change: [id: number] }>()`). Its constraints, each enforced
//! here:
//!
//! 1. It may be called **at most once** per `<script setup>` block; every call
//!    is reported otherwise.
//! 2. It may not be given **both** a type argument and a runtime argument: the
//!    runtime and type declarations are mutually exclusive.
//! 3. Its runtime argument may not **reference locally-declared variables**:
//!    `defineEmits` is hoisted above the component setup, so any binding declared
//!    in the same block (an `enum` included) is not yet initialized when the
//!    macro runs. References are resolved through scope: a type position and a
//!    parameter of a function inside the argument are not local references.
//! 4. A call with neither declares no event, unless the plain `<script>`'s
//!    `export default {}` has an `emits` option; a call next to such an option
//!    declares the events twice.
//!
//! Mirrors [`vue/valid-define-emits`](https://eslint.vuejs.org/rules/valid-define-emits.html).
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! const local = ['change']
//! defineEmits(local)                       // references a local variable
//! defineEmits<Emits>(['change'])           // both type and runtime args
//! defineEmits(['change'])
//! defineEmits(['update'])                  // called more than once
//! ```
//!
//! ### Valid
//! ```ts
//! defineEmits(['change', 'update'])
//! defineEmits<{ change: [id: number] }>()
//! ```

use super::macro_validity::{Finding, check};
use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::Program;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/valid-define-emits",
    description: "Enforce valid defineEmits() usage (no type+runtime args, no local references, single call)",
    default_severity: Severity::Error,
};

/// Enforce valid `defineEmits` usage.
pub struct ValidDefineEmits;

impl ScriptRule for ValidDefineEmits {
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
        for finding in check(program, sfc.sibling_program, "defineEmits", "emits") {
            let (span, message, help) = match finding {
                Finding::Multiple(span) => (
                    span,
                    "defineEmits() has already been called.",
                    "Merge all emitted events into a single defineEmits() call; \
                     only one is allowed per <script setup> block.",
                ),
                Finding::HasTypeAndArg(span) => (
                    span,
                    "defineEmits() cannot accept both a type argument and a runtime argument.",
                    "Declare emitted events either with the type argument \
                     (`defineEmits<{ change: [id: number] }>()`) or with a runtime argument \
                     (`defineEmits(['change'])`), not both.",
                ),
                Finding::ReferencingLocally(span) => (
                    span,
                    "defineEmits() cannot reference locally-declared variables.",
                    "defineEmits() is hoisted above the component setup, so its argument \
                     can only use imports or literals, not locally-declared variables.",
                ),
                Finding::NotDefined(span) => (
                    span,
                    "defineEmits() does not define any custom events.",
                    "Declare the events with a type argument or a runtime argument.",
                ),
                Finding::DefinedInBoth(span) => (
                    span,
                    "Custom events are defined in both defineEmits() and export default {}.",
                    "Declare the events in one place.",
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
    use super::ValidDefineEmits;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(ValidDefineEmits));
        linter
    }

    #[test]
    fn test_valid_runtime_array() {
        let result = create_linter().lint("defineEmits(['change', 'update'])", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_type_only() {
        let result = create_linter().lint("defineEmits<{ change: [id: number] }>()", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_runtime_references_import() {
        // An imported binding is not a local declaration, so referencing it is fine.
        let result =
            create_linter().lint("import { events } from './events'\ndefineEmits(events)", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_no_define_emits() {
        let result = create_linter().lint("const props = defineProps<{ a: number }>()", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_assigned_to_binding() {
        let result = create_linter().lint("const emit = defineEmits(['change'])", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_type_and_runtime_argument() {
        let result = create_linter().lint("defineEmits<Emits>(['change'])", 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_references_local_variable() {
        let result = create_linter().lint("const local = ['change']\ndefineEmits(local)", 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_references_local_function() {
        let result = create_linter().lint("function make() {}\ndefineEmits({ change: make })", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_references_destructured_local() {
        let result = create_linter().lint("const { a } = obj\ndefineEmits([a])", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_duplicate_call() {
        let result = create_linter().lint("defineEmits(['change'])\ndefineEmits(['update'])", 0);
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn test_no_warn_nested_define_emits_in_function() {
        // Only the macro itself is checked; an identifier named `defineEmits`
        // used inside a function body is not flagged for local references.
        let result = create_linter().lint("function make() { return defineEmits(['change']) }", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_offset_applied() {
        let result = create_linter().lint("defineEmits<Emits>(['change'])", 30);
        assert_eq!(result.error_count, 1);
        assert_eq!(result.diagnostics[0].start, 30);
    }
}
