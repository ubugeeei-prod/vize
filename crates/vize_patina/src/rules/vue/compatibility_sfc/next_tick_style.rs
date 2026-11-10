//! Promise-style `nextTick` usage.

use super::{absolute_range, collect_script_programs};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{CallExpression, Expression};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};

static META: RuleMeta = RuleMeta {
    name: "vue/next-tick-style",
    description: "Enforce Promise-style nextTick usage",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/next-tick-style`; the upstream default is Promise style.
pub struct NextTickStyle;

impl Rule for NextTickStyle {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let mut findings = Vec::new();
        collect_script_programs(ctx, |program, offset, source| {
            let mut visitor = NextTickCalls {
                offset,
                source,
                findings: Vec::new(),
            };
            visitor.visit_program(program);
            findings.extend(visitor.findings);
        });
        for finding in findings {
            ctx.warn_at_with_help(
                "Use the Promise returned by `nextTick`",
                finding,
                "Await `nextTick()` or chain `.then(...)` instead of passing a callback",
            );
        }
    }
}

struct NextTickCalls<'s> {
    offset: u32,
    source: &'s str,
    findings: Vec<ByteRange>,
}

impl<'a> Visit<'a> for NextTickCalls<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_next_tick_call(call)
            && (!call.arguments.is_empty() || !promise_is_consumed(self.source, call.span))
        {
            self.findings
                .push(absolute_range(self.offset, call.callee.span()));
        }
        walk::walk_call_expression(self, call);
    }
}

fn is_next_tick_call(call: &CallExpression<'_>) -> bool {
    match call.callee.get_inner_expression() {
        Expression::Identifier(identifier) => identifier.name == "nextTick",
        Expression::StaticMemberExpression(member) => {
            member.property.name == "$nextTick"
                || (member.property.name == "nextTick"
                    && matches!(
                        member.object.get_inner_expression(),
                        Expression::Identifier(identifier) if identifier.name == "Vue"
                    ))
        }
        _ => false,
    }
}

fn promise_is_consumed(source: &str, span: Span) -> bool {
    source
        .get(..span.start as usize)
        .is_some_and(|prefix| prefix.trim_end().ends_with("await"))
        || source
            .get(span.end as usize..)
            .is_some_and(|suffix| suffix.trim_start().starts_with(".then"))
}
