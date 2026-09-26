//! [`MarkupAttribute`]: a written attribute.

use super::jsx_names::{
    jsx_attribute_name, jsx_attribute_ref, jsx_static_value, jsx_value_is_dynamic,
};
use super::l2::L2Markup;
use super::l2::surface::{attr_span, attr_value};
use super::{l2_range, loc_to_range, span_to_range};
use crate::ir::ByteRange;
use oxc_ast::ast::JSXAttribute;
use std::marker::PhantomData;
use vize_l2::op::Attribute;
use vize_relief::AttributeNode;

#[derive(Clone, Copy)]
enum MarkupAttributeInner<'a> {
    Relief(&'a AttributeNode<'a>),
    Jsx {
        node: *const JSXAttribute<'a>,
        offset: u32,
    },
    L2 {
        attribute: &'a Attribute<'a>,
        doc: &'a L2Markup<'a>,
    },
    Surface {
        attr: &'a vize_l1::Attribute<'a>,
        doc: &'a L2Markup<'a>,
        name: &'a str,
    },
}

/// Static attribute view.
#[derive(Clone, Copy)]
pub struct MarkupAttribute<'a> {
    inner: MarkupAttributeInner<'a>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> MarkupAttribute<'a> {
    const fn from_inner(inner: MarkupAttributeInner<'a>) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    pub(super) const fn from_relief(node: &'a AttributeNode) -> Self {
        Self::from_inner(MarkupAttributeInner::Relief(node))
    }

    pub(super) const fn from_jsx(node: *const JSXAttribute<'a>, offset: u32) -> Self {
        Self::from_inner(MarkupAttributeInner::Jsx { node, offset })
    }

    pub(super) const fn from_l2(attribute: &'a Attribute<'a>, doc: &'a L2Markup<'a>) -> Self {
        Self::from_inner(MarkupAttributeInner::L2 { attribute, doc })
    }

    pub(super) const fn from_surface(
        attr: &'a vize_l1::Attribute<'a>,
        doc: &'a L2Markup<'a>,
    ) -> Self {
        Self::from_inner(MarkupAttributeInner::Surface {
            attr,
            doc,
            name: attr.name.text,
        })
    }

    pub(super) const fn from_authored(
        attr: &'a vize_l1::Attribute<'a>,
        doc: &'a L2Markup<'a>,
        name: &'a str,
    ) -> Self {
        Self::from_inner(MarkupAttributeInner::Surface { attr, doc, name })
    }

    /// Attribute name as written in source.
    pub fn name(&self) -> &str {
        match self.inner {
            MarkupAttributeInner::Relief(node) => node.name,
            MarkupAttributeInner::Jsx { node, .. } => {
                jsx_attribute_name(&jsx_attribute_ref(node).name)
            }
            MarkupAttributeInner::L2 { attribute, .. } => attribute.name,
            MarkupAttributeInner::Surface { name, .. } => name,
        }
    }

    /// Whether this attribute matches a normalized HTML attribute name.
    pub fn name_eq(&self, expected: &str) -> bool {
        self.name().eq_ignore_ascii_case(expected)
    }

    /// Attribute value when statically present.
    pub fn value(&self) -> Option<&'a str> {
        match self.inner {
            MarkupAttributeInner::Relief(node) => node.value.as_ref().map(|value| value.content),
            MarkupAttributeInner::Jsx { node, .. } => jsx_static_value(jsx_attribute_ref(node)),
            MarkupAttributeInner::L2 { attribute, doc } => {
                attribute.value.map(|value| doc.decode_attribute(value))
            }
            MarkupAttributeInner::Surface { attr, doc, .. } => {
                attr_value(attr).map(|value| doc.decode_attribute(value))
            }
        }
    }

    /// Whether the attribute value is dynamic.
    pub fn is_dynamic(&self) -> bool {
        match self.inner {
            MarkupAttributeInner::Jsx { node, .. } => {
                jsx_value_is_dynamic(jsx_attribute_ref(node).value.as_ref())
            }
            MarkupAttributeInner::Relief(_)
            | MarkupAttributeInner::L2 { .. }
            | MarkupAttributeInner::Surface { .. } => false,
        }
    }

    /// Attribute byte range.
    pub fn range(&self) -> ByteRange {
        match self.inner {
            MarkupAttributeInner::Relief(node) => loc_to_range(&node.loc),
            MarkupAttributeInner::Jsx { node, offset } => {
                span_to_range(jsx_attribute_ref(node).span, offset)
            }
            MarkupAttributeInner::L2 { attribute, .. } => l2_range(attribute.span),
            MarkupAttributeInner::Surface { attr, doc, .. } => {
                l2_range(attr_span(doc.source, attr))
            }
        }
    }
}
