//! Lowering JSX children into Vize template child nodes.

use oxc_ast::ast::{JSXChild, JSXExpression, JSXExpressionContainer, JSXSpreadChild};
use oxc_span::GetSpan;
use vize_carton::{Box, Vec};
use vize_relief::ast::{InterpolationNode, TemplateChildNode, TextNode};

use super::Lowerer;

impl<'a, 'm, 's> Lowerer<'a, 'm, 's> {
    /// Lower a list of JSX children, dropping whitespace-only text.
    pub(crate) fn lower_children(
        &mut self,
        children: &[JSXChild<'_>],
    ) -> Vec<'a, TemplateChildNode<'a>> {
        let mut out = Vec::new_in(self.bump());
        for child in children {
            if let Some(node) = self.lower_child(child) {
                out.push(node);
            }
        }
        out
    }

    fn lower_child(&mut self, child: &JSXChild<'_>) -> Option<TemplateChildNode<'a>> {
        match child {
            JSXChild::Text(text) => self.lower_text(text),
            JSXChild::Element(element) => {
                // A `<style scoped>` block is extracted at compile time (#1495)
                // and must not become an element vnode; drop it from the
                // rendered children once captured.
                if self.try_extract_scoped_style(element) {
                    return None;
                }
                Some(TemplateChildNode::Element(Box::new_in(
                    self.lower_element_node(element),
                    self.bump(),
                )))
            }
            JSXChild::Fragment(fragment) => Some(TemplateChildNode::Element(Box::new_in(
                self.lower_fragment_node(fragment),
                self.bump(),
            ))),
            JSXChild::ExpressionContainer(container) => self.lower_child_container(container),
            JSXChild::Spread(spread) => Some(self.lower_spread_child(spread)),
        }
    }

    fn lower_child_container(
        &mut self,
        container: &JSXExpressionContainer<'_>,
    ) -> Option<TemplateChildNode<'a>> {
        match &container.expression {
            // `{}` / `{/* comment */}` produce nothing.
            JSXExpression::EmptyExpression(_) => None,
            // `{'literal'}` lowers to plain text, covering the explicit-space
            // idiom `{' '}`.
            JSXExpression::StringLiteral(string) => Some(TemplateChildNode::Text(Box::new_in(
                TextNode::new(string.value.as_str(), self.mapper().location(string.span)),
                self.bump(),
            ))),
            expression => {
                // Recognize JSX control-flow idioms (`cond && <X/>`,
                // `cond ? <A/> : <B/>`, `items.map(i => <li/>)`) and synthesize
                // real v-if / v-for relief nodes. Anything unrecognized returns
                // `None` and falls through to plain interpolation.
                if let Some(node) = self.lower_control_flow_child(expression, container.span) {
                    return Some(node);
                }
                let content = self.dyn_expr(expression.span());
                Some(self.interpolation(content, container.span))
            }
        }
    }

    /// `{...children}` keeps the spread argument as an interpolation expression.
    fn lower_spread_child(&mut self, spread: &JSXSpreadChild<'_>) -> TemplateChildNode<'a> {
        let content = self.dyn_expr(spread.expression.span());
        self.interpolation(content, spread.span)
    }

    fn interpolation(
        &self,
        content: vize_relief::ast::ExpressionNode<'a>,
        span: oxc_span::Span,
    ) -> TemplateChildNode<'a> {
        let node = InterpolationNode {
            content,
            loc: self.mapper().location(span),
            // JSX interpolation is always escaped; the legacy raw-HTML flag
            // (Vue 1 triple-mustache) never applies here.
            #[cfg(feature = "legacy")]
            raw: false,
        };
        TemplateChildNode::Interpolation(Box::new_in(node, self.bump()))
    }
}
