//! Preserve the falsy result of a JSX logical-and expression (#6887).

use oxc_ast::ast::LogicalExpression;
use oxc_span::{GetSpan, Span};
use vize_l0::cstr;
use vize_relief::{ForNode, ForParseResult, IfBranchNode, IfNode, TemplateChildNode};

use super::Lowerer;

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
    pub(super) fn lower_logical_and(
        &self,
        logical: &LogicalExpression<'_>,
        child: TemplateChildNode<'a>,
        container_span: Span,
    ) -> TemplateChildNode<'a> {
        let span = logical.left.span();
        if self.is_provably_boolean(&logical.left) {
            let mut node = IfNode::new(self.bump(), self.mapper().location(container_span));
            let mut branch = IfBranchNode::new(
                self.bump(),
                Some(self.dyn_expr(span)),
                self.mapper().location(logical.span),
            );
            branch.children.push(child);
            node.branches.push(branch);
            return TemplateChildNode::If(self.boxed(node));
        }

        // The existing lexical scope emits an IIFE/block, never a rendered
        // list. Vapor binds a computed source, so a falsy-to-falsy update also
        // updates the text without evaluating the source twice per effect.
        let mut name = cstr!("_jsx_value{}", span.start);
        while self.mapper().source().contains(name.as_str()) {
            name.push('_');
        }
        let name = self.bump().alloc_str(&name);
        let mut node = IfNode::new(self.bump(), self.mapper().location(container_span));
        let mut truthy = IfBranchNode::new(
            self.bump(),
            Some(self.constant_expr(name, span)),
            self.mapper().location(logical.right.span()),
        );
        truthy.children.push(child);
        node.branches.push(truthy);

        // Only falsy numbers, strings and bigints produce text. Nullish values
        // and false retain the backend's existing empty comment. Arrays,
        // VNodes and boxed booleans are truthy and never become text here.
        let condition = cstr!(
            "typeof {name} === \"number\" || typeof {name} === \"string\" || typeof {name} === \"bigint\""
        );
        let condition = self.bump().alloc_str(&condition);
        let mut falsy = IfBranchNode::new(
            self.bump(),
            Some(self.constant_expr(condition, span)),
            self.mapper().location(span),
        );
        falsy
            .children
            .push(self.interpolation(self.constant_expr(name, span), span));
        node.branches.push(falsy);

        let mut parse_result = ForParseResult::new(
            self.dyn_expr(span),
            Some(self.constant_expr(name, span)),
            None,
            None,
        );
        parse_result.match_scope = true;
        let mut children = self.vec();
        children.push(TemplateChildNode::If(self.boxed(node)));
        TemplateChildNode::For(self.boxed(ForNode {
            source: self.dyn_expr(span),
            value_alias: Some(self.constant_expr(name, span)),
            key_alias: None,
            object_index_alias: None,
            parse_result,
            children,
            loc: self.mapper().location(container_span),
        }))
    }
}
