//! Configured ESTree node restrictions for retained template expressions.

use super::origin::Origin;
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{
    ArrayExpression, BinaryExpression, ConditionalExpression, ObjectExpression, UnaryExpression,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::Span;
use vize_l0::{String, cstr};
use vize_l1::markup::entity::EntityContext;
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, InterpolationNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-restricted-syntax",
    description: "Disallow configured syntax in template expressions",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Restrict selected OXC AST node names in template expressions.
#[derive(Default)]
pub struct NoRestrictedSyntax {
    selectors: Vec<String>,
}

impl NoRestrictedSyntax {
    /// Create a rule from simple ESTree node-type selectors.
    pub fn new(selectors: Vec<String>) -> Self {
        Self { selectors }
    }
}

impl Rule for NoRestrictedSyntax {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_interpolation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        interpolation: &InterpolationNode<'a>,
    ) {
        check_restricted_expression(
            ctx,
            &interpolation.content,
            &self.selectors,
            EntityContext::Text,
        );
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if let Some(expression) = &directive.exp {
            check_restricted_expression(ctx, expression, &self.selectors, EntityContext::Attribute);
        }
    }
}

fn check_restricted_expression(
    ctx: &mut LintContext<'_>,
    expression: &ExpressionNode<'_>,
    selectors: &[String],
    entity_context: EntityContext,
) {
    if selectors.is_empty() {
        return;
    }
    let ExpressionNode::Simple(simple) = expression else {
        return;
    };
    let Some(origin) = Origin::new(ctx.source, simple.content, simple.loc.span, entity_context)
    else {
        return;
    };
    let Some(retained) = simple
        .js_ast
        .filter(|retained| retained.raw == simple.content)
        .and_then(|retained| retained.as_expression())
    else {
        return;
    };
    let mut visitor = RestrictedVisitor {
        ctx,
        origin,
        selectors,
    };
    visitor.visit_expression(retained.ast);
}

struct RestrictedVisitor<'ctx, 'source, 'selectors> {
    ctx: &'ctx mut LintContext<'source>,
    origin: Origin,
    selectors: &'selectors [String],
}

impl RestrictedVisitor<'_, '_, '_> {
    fn selected(&self, node_type: &str) -> bool {
        self.selectors.iter().any(|selector| selector == node_type)
    }

    fn report(&mut self, node_type: &str, span: Span) {
        if self.selected(node_type)
            && let Some(range) = self.origin.range(span)
        {
            self.ctx.warn_at_with_help(
                cstr!(
                    "Use of `{}` is restricted in template expressions",
                    node_type
                ),
                range,
                "Rewrite the expression without the restricted syntax",
            );
        }
    }
}

impl<'a> Visit<'a> for RestrictedVisitor<'_, '_, '_> {
    fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
        self.report("CallExpression", call.span);
        walk::walk_call_expression(self, call);
    }

    fn visit_binary_expression(&mut self, binary: &BinaryExpression<'a>) {
        self.report("BinaryExpression", binary.span);
        walk::walk_binary_expression(self, binary);
    }

    fn visit_conditional_expression(&mut self, conditional: &ConditionalExpression<'a>) {
        self.report("ConditionalExpression", conditional.span);
        walk::walk_conditional_expression(self, conditional);
    }

    fn visit_object_expression(&mut self, object: &ObjectExpression<'a>) {
        self.report("ObjectExpression", object.span);
        walk::walk_object_expression(self, object);
    }

    fn visit_array_expression(&mut self, array: &ArrayExpression<'a>) {
        self.report("ArrayExpression", array.span);
        walk::walk_array_expression(self, array);
    }

    fn visit_unary_expression(&mut self, unary: &UnaryExpression<'a>) {
        self.report("UnaryExpression", unary.span);
        walk::walk_unary_expression(self, unary);
    }
}
