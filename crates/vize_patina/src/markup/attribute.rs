//! [`MarkupAttribute`]: a written attribute.

use super::jsx_names::{
    jsx_attribute_name, jsx_attribute_ref, jsx_static_value, jsx_value_is_dynamic,
};
use super::s2::S2Markup;
use super::s2::surface::{attr_span, attr_value};
use super::{loc_to_range, s2_range, span_to_range};
use crate::ir::ByteRange;
use oxc_ast::ast::JSXAttribute;
use std::marker::PhantomData;
use vize_relief::AttributeNode;
use vize_s2::op::Attribute;

#[derive(Clone, Copy)]
enum MarkupAttributeInner<'a> {
    Relief(&'a AttributeNode<'a>),
    Jsx {
        node: *const JSXAttribute<'a>,
        offset: u32,
    },
    S2 {
        attribute: &'a Attribute<'a>,
        doc: &'a S2Markup<'a>,
    },
    Surface {
        attr: &'a vize_s1::Attribute<'a>,
        doc: &'a S2Markup<'a>,
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

    pub(super) const fn from_s2(attribute: &'a Attribute<'a>, doc: &'a S2Markup<'a>) -> Self {
        Self::from_inner(MarkupAttributeInner::S2 { attribute, doc })
    }

    pub(super) const fn from_surface(
        attr: &'a vize_s1::Attribute<'a>,
        doc: &'a S2Markup<'a>,
    ) -> Self {
        Self::from_inner(MarkupAttributeInner::Surface { attr, doc })
    }

    /// Attribute name as written in source.
    pub fn name(&self) -> &str {
        match self.inner {
            MarkupAttributeInner::Relief(node) => node.name,
            MarkupAttributeInner::Jsx { node, .. } => {
                jsx_attribute_name(&jsx_attribute_ref(node).name)
            }
            MarkupAttributeInner::S2 { attribute, .. } => attribute.name,
            MarkupAttributeInner::Surface { attr, .. } => attr.name.text,
        }
    }

    /// Whether this attribute matches a normalized HTML attribute name.
    pub fn name_eq(&self, expected: &str) -> bool {
        self.name().eq_ignore_ascii_case(expected)
    }

    /// Attribute value when statically present.
    #[inline]
    pub fn value(&self) -> Option<&'a str> {
        match self.inner {
            MarkupAttributeInner::Relief(node) => node.value.as_ref().map(|value| value.content),
            MarkupAttributeInner::Jsx { node, .. } => jsx_static_value(jsx_attribute_ref(node)),
            MarkupAttributeInner::S2 { attribute, doc } => {
                attribute.value.map(|value| doc.decode_attribute(value))
            }
            MarkupAttributeInner::Surface { attr, doc } => {
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
            | MarkupAttributeInner::S2 { .. }
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
            MarkupAttributeInner::S2 { attribute, .. } => s2_range(attribute.span),
            MarkupAttributeInner::Surface { attr, doc } => s2_range(attr_span(doc.source, attr)),
        }
    }
}
