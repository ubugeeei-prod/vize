//! Authored opening-tag windows, for formatting-shaped rules.

use super::element::{MarkupElement, MarkupElementInner};
use super::jsx_names::jsx_element_ref;
use super::l2::binding::L2Item;
use super::l2::surface::{SurfaceDirective, attr_span};
use super::{l2_range, loc_to_range, span_to_range};
use crate::ir::ByteRange;
use oxc_ast::ast::JSXAttributeItem;

impl MarkupElement<'_> {
    /// Visit authored directive ranges, including malformed structural
    /// directives that did not become a scope. Frozen `v-pre` attributes
    /// remain plain attributes, matching the parser's raw-subtree contract.
    pub fn walk_authored_directive_ranges(&self, name: &str, visitor: &mut impl FnMut(ByteRange)) {
        match self.inner {
            MarkupElementInner::Relief(node) => {
                for prop in &node.props {
                    if let vize_relief::PropNode::Directive(directive) = prop
                        && directive.name == name
                    {
                        visitor(loc_to_range(&directive.loc));
                    }
                }
            }
            MarkupElementInner::L2 {
                surface: Some(element),
                doc,
                op,
            } => {
                for attr in &element.open.attrs {
                    let span = attr_span(doc.source, attr);
                    if !op.attributes().iter().any(|kept| kept.span == span)
                        && SurfaceDirective::parse(attr.name.text)
                            .is_some_and(|directive| directive.name == name)
                    {
                        visitor(l2_range(span));
                    }
                }
            }
            MarkupElementInner::Authored {
                element,
                doc,
                frozen,
                ..
            } => {
                if !frozen {
                    for attr in &element.open.attrs {
                        if SurfaceDirective::parse(attr.name.text)
                            .is_some_and(|directive| directive.name == name)
                        {
                            visitor(l2_range(attr_span(doc.source, attr)));
                        }
                    }
                }
            }
            MarkupElementInner::L2Carrier { element, doc, .. } => {
                for attr in &element.open.attrs {
                    if SurfaceDirective::parse(attr.name.text)
                        .is_some_and(|directive| directive.name == name)
                    {
                        visitor(l2_range(attr_span(doc.source, attr)));
                    }
                }
            }
            _ => self.walk_directives(&mut |directive| {
                if directive.name_eq(name) {
                    visitor(directive.range());
                }
            }),
        }
    }

    /// Visit the byte ranges of every item written on the opening tag.
    ///
    /// Unlike [`Self::walk_bindings`], this is the *authored* surface: it
    /// includes JSX spread attributes and the structural directives the facade
    /// consumes into scopes (`v-if`, `v-for`, …). Formatting-shaped rules that
    /// only need authored source windows can use this without assigning
    /// semantic meaning to those items. L2 templates read it off L1.
    pub fn walk_opening_item_ranges(&self, visitor: &mut impl FnMut(ByteRange)) {
        match self.inner {
            MarkupElementInner::Relief(node) => {
                for prop in &node.props {
                    visitor(loc_to_range(prop.loc()));
                }
            }
            MarkupElementInner::JsxElement { node, offset } => {
                for attribute in &jsx_element_ref(node).opening_element.attributes {
                    match attribute {
                        JSXAttributeItem::Attribute(attr) => {
                            visitor(span_to_range(attr.span, offset));
                        }
                        JSXAttributeItem::SpreadAttribute(spread) => {
                            visitor(span_to_range(spread.span, offset));
                        }
                    }
                }
            }
            MarkupElementInner::JsxFragment { .. } => {}
            MarkupElementInner::L2 {
                surface: Some(element),
                doc,
                op,
            } => {
                for attr in &element.open.attrs {
                    let span = attr_span(doc.source, attr);
                    // The parser drops the `v-pre` that opens a raw subtree;
                    // L2 keeps a nested one as a frozen attribute.
                    let opening_v_pre = attr.name.text == "v-pre"
                        && !op.attributes().iter().any(|kept| kept.span == span);
                    if !opening_v_pre {
                        visitor(l2_range(span));
                    }
                }
            }
            MarkupElementInner::Authored {
                element,
                doc,
                opens_v_pre,
                ..
            } => {
                for attr in &element.open.attrs {
                    if !opens_v_pre || attr.name.text != "v-pre" {
                        visitor(l2_range(attr_span(doc.source, attr)));
                    }
                }
            }
            MarkupElementInner::L2Carrier { element, doc, .. } => {
                for attr in &element.open.attrs {
                    visitor(l2_range(attr_span(doc.source, attr)));
                }
            }
            MarkupElementInner::L2 { surface: None, .. } => {
                self.walk_l2_items(&mut |item: L2Item<'_>| visitor(l2_range(item.span())));
            }
        }
    }
}
