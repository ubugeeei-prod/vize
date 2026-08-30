use super::{MarkupElement, MarkupElementInner, jsx_element_ref, loc_to_range, span_to_range};
use crate::ir::ByteRange;
use oxc_ast::ast::JSXAttributeItem;

impl MarkupElement<'_> {
    /// Visit the byte ranges of every item written on the opening tag.
    ///
    /// Unlike [`Self::walk_bindings`], this includes JSX spread attributes.
    /// Formatting-shaped rules that only need authored source windows can use
    /// this without assigning semantic meaning to spreads.
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
        }
    }
}
