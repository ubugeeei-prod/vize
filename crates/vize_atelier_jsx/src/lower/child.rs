//! Lowering JSX children into Vize template child nodes.

use oxc_ast::ast::{JSXChild, JSXExpression, JSXExpressionContainer, JSXSpreadChild};
use oxc_span::GetSpan;
use vize_carton::{Box, Vec};
use vize_relief::{InterpolationNode, TemplateChildNode, TextNode};

use super::Lowerer;

/// `<div><><i/></></div>`. A nested fragment is lowered as an element tagged
/// `Fragment`, and the DOM backend turns a component tag into
/// `resolveComponent("Fragment")`, which resolves to nothing at runtime.
/// A fragment used as the whole render root is fine — its children become the
/// root children — so only the nested case reports.
const NESTED_FRAGMENT_UNSUPPORTED: &str = "a JSX fragment nested inside an element is not supported; it lowers to an unresolvable `Fragment` component";

/// `<div>{...items}</div>`. `@vue/babel-plugin-jsx` spreads the value into the
/// children array; the lowering has no spread child, so the array used to be
/// stringified into a single text node through `toDisplayString`.
const SPREAD_CHILD_UNSUPPORTED: &str = "spread children (`{...items}`) are not supported; the value would be stringified instead of spread";

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
            JSXChild::Fragment(fragment) => {
                self.reject(fragment.span, NESTED_FRAGMENT_UNSUPPORTED);
                Some(TemplateChildNode::Element(Box::new_in(
                    self.lower_fragment_node(fragment),
                    self.bump(),
                )))
            }
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

    /// `{...children}` keeps the spread argument as an interpolation expression,
    /// which is not what a spread means; report before doing so.
    fn lower_spread_child(&mut self, spread: &JSXSpreadChild<'_>) -> TemplateChildNode<'a> {
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
        TemplateChildNode::Interpolation(Box::new_in(node, self.bump()))
    }
}
