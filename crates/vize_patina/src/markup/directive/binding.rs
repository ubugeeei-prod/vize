//! Borrow a directive's normalized binding without re-reading its element.

use super::{MarkupDirective, MarkupDirectiveInner};
use crate::markup::MarkupBinding;
use crate::markup::l2::binding::L2Item;

impl<'a> MarkupDirective<'a> {
    pub(crate) fn as_binding(&self) -> MarkupBinding<'a> {
        match self.inner {
            MarkupDirectiveInner::Relief(node) => MarkupBinding::from_relief_directive(node),
            MarkupDirectiveInner::Jsx { node, offset } => MarkupBinding::from_jsx(node, offset),
            MarkupDirectiveInner::L2(binding) => {
                MarkupBinding::from_l2_item(L2Item::Binding(binding))
            }
            MarkupDirectiveInner::Surface { attr, doc } => {
                MarkupBinding::from_surface(attr, doc, None)
            }
        }
    }
}
