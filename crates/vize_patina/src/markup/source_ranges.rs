//! Authored opening-tag windows, for formatting-shaped rules.

use super::element::{MarkupElement, MarkupElementInner};
use super::jsx_names::jsx_element_ref;
use super::s2::binding::S2Item;
use super::s2::surface::attr_span;
use super::{loc_to_range, s2_range, span_to_range};
use crate::ir::ByteRange;
use oxc_ast::ast::JSXAttributeItem;

impl MarkupElement<'_> {
    /// Visit the byte ranges of every item written on the opening tag.
    ///
    /// Unlike [`Self::walk_bindings`], this is the *authored* surface: it
    /// includes JSX spread attributes and the structural directives the facade
    /// consumes into scopes (`v-if`, `v-for`, …). Formatting-shaped rules that
    /// only need authored source windows can use this without assigning
    /// semantic meaning to those items. S2 templates read it off S1.
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
            MarkupElementInner::S2 {
                surface: Some(element),
                doc,
                op,
            } => {
                for attr in &element.open.attrs {
                    let span = attr_span(doc.source, attr);
                    // The parser drops the `v-pre` that opens a raw subtree;
                    // S2 keeps a nested one as a frozen attribute.
                    let opening_v_pre = attr.name.text == "v-pre"
                        && !op.attributes().iter().any(|kept| kept.span == span);
                    if !opening_v_pre {
                        visitor(s2_range(span));
                    }
                }
            }
            MarkupElementInner::S2Carrier { element, doc, .. } => {
                for attr in &element.open.attrs {
                    visitor(s2_range(attr_span(doc.source, attr)));
                }
            }
            MarkupElementInner::S2 { surface: None, .. } => {
                self.walk_s2_items(&mut |item: S2Item<'_>| visitor(s2_range(item.span())));
            }
        }
    }
}
