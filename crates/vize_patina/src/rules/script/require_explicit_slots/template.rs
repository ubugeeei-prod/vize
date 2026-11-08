//! `<slot>` outlets recovered from the template AST.
//!
//! # Why the AST, and why `v-pre` still needs care
//!
//! A recovered outlet *creates* a finding, so an over-match is a false
//! positive. Most of the ways a raw scan over the template text would
//! over-match are excluded structurally by taking the evidence from the AST —
//! `<!-- <slot name="x" /> -->` is a comment node and `<p>slot name="x"</p>` is
//! a text node, and neither can ever become an [`ElementNode`].
//!
//! `v-pre` is the exception. The parser rewrites the directives and
//! interpolations of a `v-pre` region (which is why the other template-aware
//! rules get `v-pre` for free), but a `<slot>` element inside one still parses
//! to an element with [`ElementType::Slot`] — while Vue renders it as a literal
//! `<slot>` tag, not a slot outlet. The directive itself is *removed* from
//! `props` by the parser, so the only remaining evidence of it is the element's
//! start tag, which is what [`start_tag_has_v_pre`] inspects. A start tag that
//! merely mentions `v-pre` inside some other attribute's value is treated as
//! `v-pre` too; that suppresses a report, the safe direction here.
//!
//! # Direction of error
//!
//! * **Over-match** would be a false positive, so a `<slot>` whose `:name` is
//!   not a string literal is skipped. Other static outlets remain checkable.
//! * **Under-match** loses a report: a `<slot>` in a template the parser could
//!   not read, or one reached only through a dynamic name, is simply not
//!   checked.

use crate::rules::opinionated::vue::authored_element::closing;
use oxc_ast::ast::Expression;
use vize_l0::CompactString;
use vize_relief::{
    ElementNode, ElementType, ExpressionNode, PropNode, RootNode, TemplateChildNode,
};

/// A `<slot>` outlet and the start-tag range to report it at, relative to the
/// template block.
pub(super) struct RenderedSlot {
    pub(super) name: CompactString,
    pub(super) start: u32,
    pub(super) end: u32,
}

/// Every `<slot>` outlet the template renders.
pub(super) enum RenderedSlots {
    /// Every outlet name is statically known.
    Known(Vec<RenderedSlot>),
}

/// Collect the `<slot>` outlets of `root`. `source` is the raw template block
/// text the AST was parsed from, used only for the `v-pre` start-tag test.
pub(super) fn collect_rendered_slots(root: &RootNode<'_>, source: &str) -> RenderedSlots {
    let mut collector = SlotCollector {
        source,
        slots: Vec::new(),
    };
    collector.walk(&root.children, false);
    RenderedSlots::Known(collector.slots)
}

struct SlotCollector<'source> {
    source: &'source str,
    slots: Vec<RenderedSlot>,
}

impl SlotCollector<'_> {
    fn walk(&mut self, children: &[TemplateChildNode<'_>], in_v_pre: bool) {
        for child in children {
            match child {
                TemplateChildNode::Element(element) => self.visit_element(element, in_v_pre),
                // `v-if` / `v-for` are still plain directives in the parse this
                // rule reads, so these arms only matter if a transformed AST is
                // ever passed in. Walking them keeps that case correct.
                TemplateChildNode::If(if_node) => {
                    for branch in if_node.branches.iter() {
                        self.walk(&branch.children, in_v_pre);
                    }
                }
                TemplateChildNode::For(for_node) => self.walk(&for_node.children, in_v_pre),
                _ => {}
            }
        }
    }

    fn visit_element(&mut self, element: &ElementNode<'_>, in_v_pre: bool) {
        let in_v_pre = in_v_pre || start_tag_has_v_pre(self.source, element);
        if !in_v_pre && element.tag_type == ElementType::Slot {
            self.record(element);
        }
        self.walk(&element.children, in_v_pre);
    }

    /// Record one `<slot>` outlet, or mark the whole scan dynamic.
    fn record(&mut self, element: &ElementNode<'_>) {
        let mut name: Option<&str> = None;
        for prop in element.props.iter() {
            match prop {
                // `<slot name>` with no value renders the default slot, so an
                // attribute without a value contributes nothing.
                PropNode::Attribute(attribute) if attribute.name == "name" => {
                    if let Some(value) = attribute.value.as_ref() {
                        name = Some(value.content);
                    }
                }
                PropNode::Directive(directive) if directive.name == "bind" => {
                    match directive.arg.as_ref() {
                        Some(arg) if argument_name(arg) == Some("name") => {
                            let Some(static_name) =
                                directive.exp.as_ref().and_then(static_string_expression)
                            else {
                                // This outlet is dynamic; eslint-plugin-vue
                                // skips it but keeps checking sibling outlets.
                                return;
                            };
                            name = Some(static_name);
                        }
                        Some(_) => {}
                        // An argument-less `v-bind` is not a `name` attribute
                        // to the upstream rule, so this remains the default
                        // outlet unless an explicit name is also present.
                        None => {}
                    }
                }
                _ => {}
            }
        }
        self.slots.push(RenderedSlot {
            name: CompactString::new(name.unwrap_or("default")),
            start: element.loc.span.start,
            end: if element.is_self_closing {
                element.loc.span.end
            } else {
                closing(self.source, element)
                    .map(|range| range.end as u32)
                    .unwrap_or(element.loc.span.end)
            },
        });
    }
}

fn static_string_expression<'a>(expression: &'a ExpressionNode<'a>) -> Option<&'a str> {
    let ExpressionNode::Simple(simple) = expression else {
        return None;
    };
    let parsed = simple.js_ast?.as_expression()?;
    match parsed.ast.get_inner_expression() {
        Expression::StringLiteral(literal) => Some(literal.value.as_str()),
        Expression::TemplateLiteral(template) => {
            template.single_quasi().map(|value| value.as_str())
        }
        _ => None,
    }
}

/// The static name of a directive argument, when it has one.
fn argument_name<'a>(arg: &'a ExpressionNode<'a>) -> Option<&'a str> {
    match arg {
        ExpressionNode::Simple(simple) if simple.is_static => Some(simple.content),
        _ => None,
    }
}

/// Whether the element's start tag carries a `v-pre` attribute.
///
/// The parser deletes the `v-pre` directive from `props`, so the start-tag text
/// is the only place it survives. A `v-pre` token is accepted only where an
/// attribute name can start (after whitespace) and end (before whitespace, `=`,
/// `/` or `>`); a mention inside another attribute's value can still match,
/// which suppresses a report rather than inventing one.
fn start_tag_has_v_pre(source: &str, element: &ElementNode<'_>) -> bool {
    let start = element.loc.span.start as usize;
    let end = element.loc.span.end as usize;
    let Some(start_tag) = source.get(start..end) else {
        return false;
    };
    let bytes = start_tag.as_bytes();
    start_tag.match_indices("v-pre").any(|(index, _)| {
        let before_ok = index
            .checked_sub(1)
            .is_none_or(|i| bytes.get(i).is_some_and(u8::is_ascii_whitespace));
        let after_ok = bytes
            .get(index + "v-pre".len())
            .is_none_or(|byte| byte.is_ascii_whitespace() || matches!(byte, b'=' | b'/' | b'>'));
        before_ok && after_ok
    })
}
