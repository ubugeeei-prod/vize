//! script/no-deprecated-dollar-scopedslots-api
//!
//! Disallow the `$scopedSlots` instance property, which was removed in Vue 3.
//!
//! ## Rationale
//!
//! In Vue 2, `this.$scopedSlots` exposed scoped slots while `this.$slots`
//! exposed only non-scoped slots. Vue 3 unified the two: every slot is now a
//! function, so `this.$scopedSlots` was removed and `this.$slots` should be
//! used instead. This is an opt-in Vue 2 -> 3 migration rule.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! const header = this.$scopedSlots.header
//! const footer = ctx.$scopedSlots.footer
//! render($scopedSlots.default)
//! ```
//!
//! ### Valid
//! ```ts
//! const header = this.$slots.header
//! const footer = ctx.$slots.footer
//! render($slots.default)
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    ComputedMemberExpression, Expression, IdentifierReference, Program, StaticMemberExpression,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_computed_member_expression, walk_identifier_reference, walk_static_member_expression,
    },
};
use oxc_span::Span;

/// The removed Vue 2 instance property this rule reports.
const SCOPED_SLOTS: &str = "$scopedSlots";

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-deprecated-dollar-scopedslots-api",
    description: "Disallow the $scopedSlots instance property removed in Vue 3 (use $slots)",
    default_severity: Severity::Error,
};

/// Disallow the `$scopedSlots` instance property (removed in Vue 3).
pub struct NoDeprecatedDollarScopedSlotsApi;

impl ScriptRule for NoDeprecatedDollarScopedSlotsApi {
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
        let mut visitor = NoDeprecatedDollarScopedSlotsApiVisitor { offset, result };
        visitor.visit_program(program);
    }
}

struct NoDeprecatedDollarScopedSlotsApiVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for NoDeprecatedDollarScopedSlotsApiVisitor<'_> {
    fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
        // `this.$scopedSlots`, `ctx.$scopedSlots`, etc. Report the whole
        // member-access span so the fix target covers `<object>.$scopedSlots`.
        if it.property.name.as_str() == SCOPED_SLOTS {
            self.push_diagnostic(it.span);
        }

        walk_static_member_expression(self, it);
    }

    fn visit_computed_member_expression(&mut self, it: &ComputedMemberExpression<'a>) {
        // `this["$scopedSlots"]` and friends: only a string-literal key can name
        // the removed property statically.
        if let Expression::StringLiteral(key) = &it.expression
            && key.value.as_str() == SCOPED_SLOTS
        {
            self.push_diagnostic(it.span);
        }

        walk_computed_member_expression(self, it);
    }

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        // A bare `$scopedSlots` reference (e.g. destructured off the instance).
        // The property of a static member access is an `IdentifierName`, not an
        // `IdentifierReference`, so this never double-counts `obj.$scopedSlots`.
        if it.name.as_str() == SCOPED_SLOTS {
            self.push_diagnostic(it.span);
        }

        walk_identifier_reference(self, it);
    }
}

impl NoDeprecatedDollarScopedSlotsApiVisitor<'_> {
    fn push_diagnostic(&mut self, span: Span) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        self.result.add_diagnostic(
            LintDiagnostic::error(
                META.name,
                "`$scopedSlots` was removed in Vue 3",
                start,
                end,
            )
            .with_help(
                "Replace `$scopedSlots` with `$slots`. In Vue 3 every slot is a function, so the separate scoped-slots property no longer exists.",
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::NoDeprecatedDollarScopedSlotsApi;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoDeprecatedDollarScopedSlotsApi));
        linter
    }

    fn lint(source: &str) -> crate::rules::script::ScriptLintResult {
        create_linter().lint(source, 0)
    }

    #[test]
    fn test_valid_dollar_slots() {
        // The Vue 3 replacement must not be flagged.
        let result = lint("const header = this.$slots.header");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_unrelated_members() {
        let source = r#"
const a = this.$attrs
const b = this.$props
const c = this.$emit
const slots = useSlots()
"#;
        let result = lint(source);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_unrelated_property_name() {
        // A property merely containing the substring must not match.
        let result = lint("const x = this.$scopedSlotsHelper");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_this_scoped_slots() {
        let result = lint("const header = this.$scopedSlots.header");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_ctx_scoped_slots() {
        let result = lint("const footer = ctx.$scopedSlots.footer");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_bare_identifier_reference() {
        let source = r#"
const { $scopedSlots } = this
render($scopedSlots.default)
"#;
        // One binding-pattern occurrence (not a reference) plus one reference
        // usage; the destructuring target is a binding identifier, so only the
        // `render(...)` argument is an `IdentifierReference`.
        let result = lint(source);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_computed_string_key() {
        let result = lint(r#"const header = this["$scopedSlots"].header"#);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_multiple_occurrences() {
        let source = r#"
const a = this.$scopedSlots.header
const b = ctx.$scopedSlots.footer
"#;
        let result = lint(source);
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn test_reports_member_span() {
        let result = lint("const header = this.$scopedSlots.header");
        assert_eq!(result.diagnostics.len(), 1);
        let diag = &result.diagnostics[0];
        // Span covers `this.$scopedSlots`, not the trailing `.header` access.
        let start = source_index(
            "const header = this.$scopedSlots.header",
            "this.$scopedSlots",
        );
        assert_eq!(diag.start, start as u32);
        assert_eq!(diag.end, (start + "this.$scopedSlots".len()) as u32);
    }

    fn source_index(haystack: &str, needle: &str) -> usize {
        haystack.find(needle).expect("needle present in source")
    }
}
