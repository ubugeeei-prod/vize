//! script/require-explicit-slots
//!
//! Require slots consumed in a TypeScript `<script setup>` to be explicitly
//! typed with `defineSlots<...>()`.
//!
//! When a component reads its slots programmatically — via `useSlots()` — but
//! never declares them with `defineSlots`, the slot names and their prop types
//! are invisible to the type-checker and to consumers of the component. The
//! recommended pattern is to declare slots up front with the type-only
//! `defineSlots<{ ... }>()` macro so the slot contract is explicit and typed.
//!
//! This rule fires only when **all** of the following hold for a single
//! `<script setup>` block:
//!
//! * the block contains TypeScript syntax (a type annotation, interface, type
//!   alias, `defineProps<T>()`, etc.). Because a script rule does not receive
//!   the SFC `lang` attribute, the presence of TS syntax is used as a sound
//!   proxy for `lang="ts"`, matching `vue/require-explicit-slots`, which only
//!   runs for TypeScript SFCs. A block with no TS syntax at all is treated as
//!   JavaScript and never flagged.
//! * slots are clearly consumed: there is at least one `useSlots()` call.
//! * there is **no** `defineSlots(...)` / `defineSlots<...>()` declaration in
//!   the same block.
//!
//! The rule is deliberately conservative — it flags only the clear
//! "useSlots without defineSlots" case to avoid false positives. The report is
//! anchored at the first `useSlots()` call so the fix (adding a `defineSlots`
//! declaration) is obvious.
//!
//! Mirrors [`vue/require-explicit-slots`](https://eslint.vuejs.org/rules/require-explicit-slots.html),
//! which applies to TypeScript only. That rule additionally validates `<slot>`
//! usage against the declared slots in the template; this port covers the
//! script-side `useSlots`-without-`defineSlots` subset, which is the portion
//! observable from a single script block.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! const props = defineProps<{ id: number }>()
//! const slots = useSlots()
//! ```
//!
//! ### Valid
//! ```ts
//! defineSlots<{ default(props: { msg: string }): unknown }>()
//! const slots = useSlots()
//! ```

use oxc_ast::ast::{CallExpression, Expression, Program, TSType};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_ts_type},
};
use oxc_span::Span;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-explicit-slots",
    description: "Require slots consumed via useSlots() to be explicitly typed with defineSlots<...>()",
    default_severity: Severity::Warning,
};

const MESSAGE: &str =
    "Slots consumed via useSlots() must be explicitly typed with defineSlots<...>().";
const HELP: &str = "Declare the slots with the type-only macro, e.g. \
     `defineSlots<{ default(props: {}): unknown }>()`.";

/// Require `defineSlots<...>()` when slots are consumed via `useSlots()`.
pub struct RequireExplicitSlots;

impl ScriptRule for RequireExplicitSlots {
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
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let mut visitor = SlotsVisitor {
            has_ts_syntax: false,
            has_define_slots: false,
            first_use_slots: None,
        };
        visitor.visit_program(program);

        // Conservative gate: only a TypeScript block that consumes slots via
        // `useSlots()` and never declares them with `defineSlots` is flagged.
        if !visitor.has_ts_syntax || visitor.has_define_slots {
            return;
        }
        let Some(span) = visitor.first_use_slots else {
            return;
        };

        let start = offset as u32 + span.start;
        let end = offset as u32 + span.end;
        result.add_diagnostic(
            LintDiagnostic::warn(META.name, MESSAGE, start, end)
                .with_label("slots consumed here without defineSlots", start, end)
                .with_help(HELP),
        );
    }
}

struct SlotsVisitor {
    /// Whether the block contains any TypeScript-specific syntax. Used as a
    /// sound proxy for `lang="ts"` since a script rule cannot read the SFC
    /// `lang` attribute.
    has_ts_syntax: bool,
    /// Whether a `defineSlots(...)` / `defineSlots<...>()` call is present.
    has_define_slots: bool,
    /// Span of the first `useSlots()` call, if any.
    first_use_slots: Option<Span>,
}

impl<'a> Visit<'a> for SlotsVisitor {
    fn visit_ts_type(&mut self, it: &TSType<'a>) {
        // Any TypeScript type position is a definitive TS-syntax signal.
        self.has_ts_syntax = true;
        walk_ts_type(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &it.callee {
            match callee.name.as_str() {
                "defineSlots" => {
                    self.has_define_slots = true;
                    // A `defineSlots<T>()` call carries a type argument, which is
                    // itself TS syntax; mark it so a block whose only TS token is
                    // the slots declaration is still recognised as TypeScript.
                    if it.type_arguments.is_some() {
                        self.has_ts_syntax = true;
                    }
                }
                "useSlots" if self.first_use_slots.is_none() => {
                    self.first_use_slots = Some(it.span);
                }
                _ => {}
            }
        }
        walk_call_expression(self, it);
    }
}

#[cfg(test)]
mod tests {
    use super::RequireExplicitSlots;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(RequireExplicitSlots));
        linter
    }

    #[test]
    fn test_invalid_use_slots_without_define_slots() {
        // TS syntax present (defineProps<T>()), useSlots() consumed, no defineSlots.
        let result = create_linter().lint(
            "const props = defineProps<{ id: number }>()\nconst slots = useSlots()",
            0,
        );
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_use_slots_with_type_annotation() {
        // TS signalled by a plain type annotation.
        let result = create_linter().lint("const n: number = 1\nconst slots = useSlots()", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_use_slots_with_interface() {
        let result =
            create_linter().lint("interface Props { id: number }\nconst s = useSlots()", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_use_slots_called_without_assignment() {
        // A bare `useSlots()` expression statement still counts as consumption.
        let result = create_linter().lint("const x: string = ''\nuseSlots()", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_define_slots_typed() {
        let result = create_linter().lint(
            "defineSlots<{ default(props: { msg: string }): unknown }>()\nconst slots = useSlots()",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_define_slots_alone_satisfies() {
        // `defineSlots` present (even before `useSlots`) means slots are declared.
        let result = create_linter().lint(
            "const props = defineProps<{ id: number }>()\ndefineSlots<{ default(): unknown }>()\nconst slots = useSlots()",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_no_use_slots() {
        // No slot consumption at all.
        let result = create_linter().lint("const props = defineProps<{ id: number }>()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_javascript_use_slots() {
        // No TypeScript syntax anywhere => treated as JS, not flagged. In JS the
        // type-only `defineSlots<T>()` fix is not available, so flagging would be
        // a false positive.
        let result = create_linter().lint("const slots = useSlots()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_javascript_use_slots_with_props() {
        // Runtime `defineProps([...])` carries no TS syntax => still JS.
        let result = create_linter().lint(
            "const props = defineProps(['id'])\nconst slots = useSlots()",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_define_slots_typed_only_ts_token() {
        // The only TS token is the `defineSlots<T>()` type argument; since
        // `defineSlots` is present the block is valid regardless.
        let result = create_linter().lint(
            "defineSlots<{ header(): unknown }>()\nconst slots = useSlots()",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_no_warn_use_slots_in_string_literal() {
        // The pattern inside a string literal must not be flagged.
        let result = create_linter().lint("const code: string = \"const s = useSlots()\"", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_multiple_use_slots_reports_once_at_first() {
        let source = "const x: number = 0\nconst a = useSlots()\nconst b = useSlots()";
        let result = create_linter().lint(source, 0);
        assert_eq!(result.warning_count, 1);
        let first = source.find("useSlots()").unwrap() as u32;
        assert_eq!(result.diagnostics[0].start, first);
    }

    #[test]
    fn test_offset_applied() {
        let source = "const x: number = 0\nconst slots = useSlots()";
        let result = create_linter().lint(source, 100);
        assert_eq!(result.warning_count, 1);
        let call_start = source.find("useSlots()").unwrap() as u32 + 100;
        assert_eq!(result.diagnostics[0].start, call_start);
    }
}
