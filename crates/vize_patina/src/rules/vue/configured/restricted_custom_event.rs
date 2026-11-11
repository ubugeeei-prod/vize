//! `vue/no-restricted-custom-event`.
//!
//! Reports template `$emit` calls. `emit` from a `setup()` parameter and
//! `this.$emit` are eslint-plugin-vue checks this port does not run, and the
//! binding `defineEmits` returns stays quiet.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{Argument, CallExpression, Expression};
use oxc_ast_visit::{Visit, walk as walk_ast};
use vize_l0::config::RestrictedNames;
use vize_l0::{String, cstr};
use vize_relief::{ExpressionNode, PropNode, RootNode, SimpleExpressionNode, TemplateChildNode};

use super::template_walk::walk;
use super::{hits, listed, report_template};

static META: RuleMeta = RuleMeta {
    name: "vue/no-restricted-custom-event",
    description: "disallow specific custom event",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/no-restricted-custom-event`.
#[derive(Default)]
pub struct NoRestrictedCustomEvent {
    names: Vec<String>,
}

impl NoRestrictedCustomEvent {
    /// Build the rule from `linter.ruleOptions["vue/no-restricted-custom-event"]`.
    pub(crate) fn from_options(options: &RestrictedNames) -> Self {
        Self {
            names: listed(options),
        }
    }
}

impl Rule for NoRestrictedCustomEvent {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        if self.names.is_empty() {
            return;
        }
        let mut findings = Vec::new();
        walk(&root.children, 1, &mut |element, _depth| {
            for prop in &element.props {
                if let PropNode::Directive(directive) = prop
                    && let Some(ExpressionNode::Simple(expression)) = &directive.exp
                {
                    emit_findings(ctx, expression, &self.names, &mut findings);
                }
            }
            emit_children(ctx, &element.children, &self.names, &mut findings);
        });
        emit_children(ctx, &root.children, &self.names, &mut findings);
        report_template(ctx, META.name, findings);
    }
}

fn emit_children(
    ctx: &LintContext<'_>,
    children: &[TemplateChildNode<'_>],
    names: &[String],
    findings: &mut Vec<super::Finding>,
) {
    for child in children {
        if let TemplateChildNode::Interpolation(node) = child
            && let ExpressionNode::Simple(expression) = &node.content
        {
            emit_findings(ctx, expression, names, findings);
        }
    }
}

fn emit_findings(
    ctx: &LintContext<'_>,
    expression: &SimpleExpressionNode<'_>,
    names: &[String],
    findings: &mut Vec<super::Finding>,
) {
    let Some(parsed) = expression.js_ast.and_then(|ast| ast.as_expression()) else {
        return;
    };
    let Some(base) = expression_base(ctx.source, expression, parsed.raw) else {
        return;
    };
    let mut visitor = EmitVisitor {
        names,
        base,
        findings,
    };
    visitor.visit_expression(parsed.ast);
}

struct EmitVisitor<'a> {
    names: &'a [String],
    base: u32,
    findings: &'a mut Vec<super::Finding>,
}

impl<'a> Visit<'a> for EmitVisitor<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_emit(&call.callee)
            && let Some(Argument::StringLiteral(literal)) = call.arguments.first()
            && hits(self.names, literal.value.as_str())
        {
            let name = literal.value.as_str();
            self.findings.push((
                cstr!("Using `{name}` event is not allowed."),
                self.base + literal.span.start,
                self.base + literal.span.end,
            ));
        }
        walk_ast::walk_call_expression(self, call);
    }
}

fn is_emit(callee: &Expression<'_>) -> bool {
    match callee.get_inner_expression() {
        Expression::Identifier(identifier) => identifier.name == "$emit",
        _ => false,
    }
}

fn expression_base(source: &str, expression: &SimpleExpressionNode<'_>, raw: &str) -> Option<u32> {
    let start = usize::try_from(expression.loc.span.start).ok()?;
    let end = usize::try_from(expression.loc.span.end).ok()?;
    (source.get(start..end) == Some(raw)).then_some(expression.loc.span.start)
}
