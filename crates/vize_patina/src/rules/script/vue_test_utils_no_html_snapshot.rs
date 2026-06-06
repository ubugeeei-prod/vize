//! ecosystem/vue-test-utils-no-html-snapshot
//!
//! Discourage full-wrapper HTML snapshots in Vue Test Utils tests.
//!
//! `wrapper.html()` snapshots are brittle because unrelated compiler, formatter,
//! or component-stub changes rewrite large strings. Focused assertions keep
//! tests useful for agentic refactors and production maintenance.

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use memchr::memmem;
use oxc_ast::ast::{Argument, CallExpression, Expression, Program};
use oxc_ast_visit::{Visit, walk::walk_call_expression};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "ecosystem/vue-test-utils-no-html-snapshot",
    description: "Avoid snapshotting wrapper.html() in Vue Test Utils tests",
    default_severity: Severity::Warning,
};

pub struct VueTestUtilsNoHtmlSnapshot;

impl ScriptRule for VueTestUtilsNoHtmlSnapshot {
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
        let bytes = source.as_bytes();
        if memmem::find(bytes, b"toMatchSnapshot").is_none()
            || memmem::find(bytes, b".html").is_none()
        {
            return;
        }

        let mut visitor = HtmlSnapshotVisitor { offset, result };
        visitor.visit_program(program);
    }
}

struct HtmlSnapshotVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for HtmlSnapshotVisitor<'_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if is_html_snapshot_assertion(it) {
            let span = it.span;
            self.result.add_diagnostic(
                LintDiagnostic::warn(
                    META.name,
                    "Avoid wrapper.html() snapshots",
                    self.offset as u32 + span.start,
                    self.offset as u32 + span.end,
                )
                .with_help(
                    "Assert visible behavior with text, attributes, emitted events, or component state instead of snapshotting the whole rendered HTML string.",
                ),
            );
        }

        walk_call_expression(self, it);
    }
}

fn is_html_snapshot_assertion(call: &CallExpression<'_>) -> bool {
    let Expression::StaticMemberExpression(assertion) = &call.callee else {
        return false;
    };
    if assertion.property.name.as_str() != "toMatchSnapshot" {
        return false;
    }

    let Expression::CallExpression(expect_call) = &assertion.object else {
        return false;
    };
    let Expression::Identifier(callee) = &expect_call.callee else {
        return false;
    };
    if callee.name.as_str() != "expect" {
        return false;
    }

    expect_call
        .arguments
        .first()
        .is_some_and(argument_is_wrapper_html_call)
}

fn argument_is_wrapper_html_call(argument: &Argument<'_>) -> bool {
    let Argument::CallExpression(call) = argument else {
        return false;
    };

    let Expression::StaticMemberExpression(member) = &call.callee else {
        return false;
    };

    member.property.name.as_str() == "html" && call.arguments.is_empty()
}

#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod tests {
    use super::{ScriptLintResult, ScriptRule, VueTestUtilsNoHtmlSnapshot};

    #[test]
    fn accepts_text_assertion() {
        let mut result = ScriptLintResult::default();
        VueTestUtilsNoHtmlSnapshot.check(
            "expect(wrapper.text()).toContain('Saved')",
            0,
            &mut result,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_html_snapshot() {
        let mut result = ScriptLintResult::default();
        VueTestUtilsNoHtmlSnapshot.check(
            "expect(wrapper.html()).toMatchSnapshot()",
            0,
            &mut result,
        );
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }
}
