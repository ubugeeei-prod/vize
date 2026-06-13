//! script/no-deprecated-dollar-listeners-api
//!
//! Disallow the `$listeners` instance property.
//!
//! ## Rationale
//!
//! `$listeners` was removed in Vue 3. In Vue 2 it exposed the parent-scope
//! event listeners separately from `$attrs`; in Vue 3 listeners are plain
//! props prefixed with `on`, so they are merged into `$attrs`. Any code that
//! still reads `$listeners` (e.g. `this.$listeners`, `ctx.$listeners`, or a
//! bare `$listeners` reference) will not work under Vue 3 and must be migrated
//! to `$attrs`.
//!
//! This is a Vue 2 -> 3 migration rule and is opt-in.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! const handlers = this.$listeners
//! const forwarded = ctx.$listeners
//! emit('input', $listeners)
//! ```
//!
//! ### Valid
//! ```ts
//! const handlers = this.$attrs
//! const forwarded = ctx.attrs
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{IdentifierReference, Program, StaticMemberExpression};
use oxc_ast_visit::{
    Visit,
    walk::{walk_identifier_reference, walk_static_member_expression},
};
use oxc_span::Span;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-deprecated-dollar-listeners-api",
    description: "Disallow the $listeners instance property removed in Vue 3 (merged into $attrs)",
    default_severity: Severity::Error,
};

/// The deprecated instance property name removed in Vue 3.
const DOLLAR_LISTENERS: &str = "$listeners";

const MESSAGE: &str = "$listeners was removed in Vue 3 and merged into $attrs";
const HELP: &str = "Replace $listeners with $attrs. In Vue 3 listeners are passed as on-prefixed props and are \
     exposed through $attrs (or useAttrs() in the Composition API).";

/// Disallow the `$listeners` instance property (Vue 3 migration).
pub struct NoDeprecatedDollarListenersApi;

impl ScriptRule for NoDeprecatedDollarListenersApi {
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
        let mut visitor = NoDeprecatedDollarListenersApiVisitor { offset, result };
        visitor.visit_program(program);
    }
}

struct NoDeprecatedDollarListenersApiVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for NoDeprecatedDollarListenersApiVisitor<'_> {
    fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
        // Member access whose property is `$listeners`, e.g. `this.$listeners`,
        // `ctx.$listeners`, `vm.$listeners`. Report on the whole member-access
        // span so the diagnostic covers the receiver and the property.
        if it.property.name.as_str() == DOLLAR_LISTENERS {
            self.push_diagnostic(it.span);
        }

        walk_static_member_expression(self, it);
    }

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        // A bare `$listeners` reference (e.g. destructured from a context or an
        // auto-imported binding). Static member *property* names are
        // `IdentifierName`, not `IdentifierReference`, so `this.$listeners` is
        // not double-counted here.
        if it.name.as_str() == DOLLAR_LISTENERS {
            self.push_diagnostic(it.span);
        }

        walk_identifier_reference(self, it);
    }
}

impl NoDeprecatedDollarListenersApiVisitor<'_> {
    fn push_diagnostic(&mut self, span: Span) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        self.result
            .add_diagnostic(LintDiagnostic::error(META.name, MESSAGE, start, end).with_help(HELP));
    }
}

#[cfg(test)]
mod tests {
    use super::NoDeprecatedDollarListenersApi;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoDeprecatedDollarListenersApi));
        linter
    }

    #[test]
    fn test_valid_uses_attrs() {
        let linter = create_linter();
        let result = linter.lint("const handlers = this.$attrs", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_no_dollar_listeners() {
        let linter = create_linter();
        let result = linter.lint(
            r#"
import { useAttrs } from 'vue'
const attrs = useAttrs()
const onClick = attrs.onClick
"#,
            0,
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_similar_named_property_not_flagged() {
        // `$listenersCount` / `listeners` must not be confused with `$listeners`.
        let linter = create_linter();
        let result = linter.lint(
            "const a = this.$listenersCount; const b = ctx.listeners; const c = $listener",
            0,
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_dollar_attrs_and_emit_not_flagged() {
        // Sibling instance APIs that DO exist in Vue 3 must not be reported.
        let linter = create_linter();
        let result = linter.lint("this.$attrs; this.$emit('input'); this.$slots", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_this_dollar_listeners() {
        let linter = create_linter();
        let result = linter.lint("const handlers = this.$listeners", 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_ctx_dollar_listeners() {
        let linter = create_linter();
        let result = linter.lint(
            "export default { setup(_, ctx) { return ctx.$listeners } }",
            0,
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_vm_dollar_listeners() {
        let linter = create_linter();
        let result = linter.lint("const forwarded = vm.$listeners", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_bare_dollar_listeners_reference() {
        let linter = create_linter();
        let result = linter.lint("emit('listeners', $listeners)", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_multiple_dollar_listeners() {
        let linter = create_linter();
        let result = linter.lint(
            r#"
const a = this.$listeners
const b = ctx.$listeners
"#,
            0,
        );
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn test_member_access_not_double_counted() {
        // `this.$listeners` is exactly one diagnostic, not two (member + ident).
        let linter = create_linter();
        let result = linter.lint("this.$listeners", 0);
        assert_eq!(result.error_count, 1);
    }
}
