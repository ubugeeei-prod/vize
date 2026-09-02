//! Lowering JSX children into Vize template child nodes.

use oxc_ast::ast::{JSXChild, JSXExpression, JSXExpressionContainer, JSXSpreadChild};
use oxc_span::GetSpan;
use vize_relief::{
    CompoundExpressionChild, CompoundExpressionNode, ExpressionNode, InterpolationNode,
    TemplateChildNode, TextNode,
};
use vize_s0::{Box, Vec};

use super::Lowerer;

/// `<div>{...items}</div>`. `@vue/babel-plugin-jsx` spreads the value into the
/// children array; the lowering has no spread child, so the array used to be
/// stringified into a single text node through `toDisplayString`.
const SPREAD_CHILD_UNSUPPORTED: &str = "spread children (`{...items}`) are not supported; the value would be stringified instead of spread";

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
    /// Lower children of an intrinsic element, preserving Babel's raw value
    /// semantics for a lone expression child in opt-in VDOM compatibility mode.
    ///
    /// Vize's native template-shaped output stringifies `{value}` and marks the
    /// vnode with `TEXT`. Babel passes the value as a child array entry instead,
    /// so it contributes no text patch flag. Component children deliberately do
    /// not use this path because they are lowered through slot synthesis.
    pub(crate) fn lower_element_children(
        &mut self,
        children: &[JSXChild<'_>],
    ) -> Vec<'a, TemplateChildNode<'a>> {
        if !self.uses_babel_vdom_compat() {
            return self.lower_children(children);
        }

        let [JSXChild::ExpressionContainer(container)] = children else {
            return self.lower_children(children);
        };
        match &container.expression {
            JSXExpression::EmptyExpression(_) | JSXExpression::StringLiteral(_) => {
                self.lower_children(children)
            }
            expression => {
                let mut out = self.vec();
                if let Some(control_flow) =
                    self.lower_control_flow_child(expression, container.span)
                {
                    out.push(control_flow);
                } else {
                    out.push(self.raw_expression_child(expression.span(), container.span));
                }
                out
            }
        }
    }

    /// Lower a list of JSX children, dropping whitespace-only text.
    pub(crate) fn lower_children(
        &mut self,
        children: &[JSXChild<'_>],
    ) -> Vec<'a, TemplateChildNode<'a>> {
        let mut out = self.vec();
        for child in children {
            self.push_child(child, &mut out);
        }
        out
    }

    /// Lower one JSX child, appending zero or more nodes to `out`. A child is
    /// not one-to-one with a node: `<style scoped>` and `{}` contribute none,
    /// and a fragment contributes its whole child list.
    fn push_child(&mut self, child: &JSXChild<'_>, out: &mut Vec<'a, TemplateChildNode<'a>>) {
        match child {
            // `<div><><i/><b/></></div>`. A fragment in child position carries
            // no props and cannot be keyed, so its children *are* the parent's
            // children at that position, in every backend. Splicing them in is
            // therefore equivalent to the nested `Fragment` vnode
            // `@vue/babel-plugin-jsx` emits, and it removes the node the DOM
            // backend used to resolve by name as `resolveComponent("Fragment")`
            // — an unresolvable component (#3421).
            JSXChild::Fragment(fragment) => {
                for nested in &fragment.children {
                    self.push_child(nested, out);
                }
            }
            JSXChild::Text(text) => out.extend(self.lower_text(text)),
            JSXChild::Element(element) => {
                // A `<style scoped>` block is extracted at compile time (#1495)
                // and must not become an element vnode; drop it from the
                // rendered children once captured.
                if self.try_extract_scoped_style(element) {
                    return;
                }
                let node = self.lower_element_node(element);
                out.push(TemplateChildNode::Element(self.boxed(node)));
            }
            JSXChild::ExpressionContainer(container) => {
                out.extend(self.lower_child_container(container));
            }
            JSXChild::Spread(spread) => {
                let node = self.lower_spread_child(spread);
                out.push(node);
            }
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
                TextNode::new(
                    self.bump().alloc_str(string.value.as_str()),
                    self.mapper().location(string.span),
                ),
                &self.bump(),
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

    /// `{...children}` keeps the spread argument as an interpolation expression,
    /// which is not what a spread means; report before doing so.
    fn lower_spread_child(&mut self, spread: &JSXSpreadChild<'_>) -> TemplateChildNode<'a> {
        if self.uses_babel_vdom_compat() {
            let ExpressionNode::Simple(expression) = self.dyn_expr(spread.expression.span()) else {
                unreachable!("span-backed JSX expressions are always simple expressions")
            };
            let mut compound =
                CompoundExpressionNode::new(self.bump(), self.mapper().location(spread.span));
            compound
                .children
                .push(CompoundExpressionChild::String("..."));
            compound
                .children
                .push(CompoundExpressionChild::Simple(expression));
            return TemplateChildNode::CompoundExpression(self.boxed(compound));
        }

        self.reject(spread.span, SPREAD_CHILD_UNSUPPORTED);
        let content = self.dyn_expr(spread.expression.span());
        self.interpolation(content, spread.span)
    }

    pub(crate) fn interpolation(
        &self,
        content: vize_relief::ExpressionNode<'a>,
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
        TemplateChildNode::Interpolation(self.boxed(node))
    }

    /// Represent a JSX child value as an unescaped expression rather than a
    /// template interpolation. The core code generator already models mixed
    /// JavaScript expressions as compound expressions; a single dynamic part
    /// is the narrowest existing IR shape that keeps this JSX-only distinction
    /// out of the public relief node surface.
    pub(crate) fn raw_expression_child(
        &self,
        expression_span: oxc_span::Span,
        container_span: oxc_span::Span,
    ) -> TemplateChildNode<'a> {
        let ExpressionNode::Simple(expression) = self.dyn_expr(expression_span) else {
            unreachable!("span-backed JSX expressions are always simple expressions")
        };
        let mut compound =
            CompoundExpressionNode::new(self.bump(), self.mapper().location(container_span));
        compound
            .children
            .push(CompoundExpressionChild::Simple(expression));
        TemplateChildNode::CompoundExpression(self.boxed(compound))
    }
}
