use super::child::Parent;
use super::{ChildrenView, ParsedComponent};
use crate::markup::{Attribute, Element};

mod names;
pub use names::{ElementClosingName, ElementNameRefusal, ElementNames};

/// Original element derived only from an authenticated child projection.
pub struct ElementView<'o, 'a> {
    component: &'o ParsedComponent<'a>,
    parent: Parent<'o, 'a>,
    ordinal: usize,
    element: &'o Element<'a>,
}
impl<'o, 'a> ElementView<'o, 'a> {
    pub(super) fn from_child(
        component: &'o ParsedComponent<'a>,
        parent: Parent<'o, 'a>,
        ordinal: usize,
        element: &'o Element<'a>,
    ) -> Self {
        Self {
            component,
            parent,
            ordinal,
            element,
        }
    }
    #[must_use]
    pub fn component(&self) -> &'o ParsedComponent<'a> {
        self.component
    }
    #[must_use]
    pub fn parent_element(&self) -> Option<&'o Element<'a>> {
        match self.parent {
            Parent::Root => None,
            Parent::Element(element) => Some(element),
        }
    }
    #[must_use]
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }
    #[must_use]
    pub fn surface(&self) -> &'o Element<'a> {
        self.element
    }
    #[must_use]
    pub fn children(&self) -> ChildrenView<'o, 'a> {
        ChildrenView::element(self.component, self.element)
    }
    #[must_use]
    pub fn attributes(&self) -> AttributesView<'o, 'a> {
        AttributesView {
            component: self.component,
            element: self.element,
            attributes: self.element.open.attrs.iter().enumerate(),
        }
    }
}

/// Actual opening-header iteration with a short original-owner borrow.
pub struct AttributesView<'o, 'a> {
    component: &'o ParsedComponent<'a>,
    element: &'o Element<'a>,
    attributes: core::iter::Enumerate<core::slice::Iter<'o, Attribute<'a>>>,
}
impl<'o, 'a> Iterator for AttributesView<'o, 'a> {
    type Item = AttributeView<'o, 'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let (ordinal, attribute) = self.attributes.next()?;
        Some(AttributeView {
            component: self.component,
            element: self.element,
            ordinal,
            attribute,
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.attributes.size_hint()
    }
}
impl ExactSizeIterator for AttributesView<'_, '_> {}

/// Full immutable original Attribute, not an arbitrary same-span carrier.
pub struct AttributeView<'o, 'a> {
    component: &'o ParsedComponent<'a>,
    element: &'o Element<'a>,
    ordinal: usize,
    attribute: &'o Attribute<'a>,
}
impl<'o, 'a> AttributeView<'o, 'a> {
    /// Borrow the same original header event for observation before consumption.
    /// The projection cannot outlive this token or change its ordinal/membership.
    #[must_use]
    pub fn reborrow(&self) -> AttributeView<'_, 'a> {
        AttributeView {
            component: self.component,
            element: self.element,
            ordinal: self.ordinal,
            attribute: self.attribute,
        }
    }
    #[must_use]
    pub fn component(&self) -> &'o ParsedComponent<'a> {
        self.component
    }
    #[must_use]
    pub fn element(&self) -> &'o Element<'a> {
        self.element
    }
    #[must_use]
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }
    #[must_use]
    pub fn surface(&self) -> &'o Attribute<'a> {
        self.attribute
    }
}
