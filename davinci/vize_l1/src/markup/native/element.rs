use super::child::Parent;
use super::{NativeChildren, NativeComponent};
use crate::{Attribute, Element};

/// Original element derived only from an authenticated child projection.
pub struct NativeElement<'o, 'a> {
    component: &'o NativeComponent<'a>,
    parent: Parent<'o, 'a>,
    ordinal: usize,
    element: &'o Element<'a>,
}
impl<'o, 'a> NativeElement<'o, 'a> {
    pub(super) fn from_child(
        component: &'o NativeComponent<'a>,
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
    pub fn component(&self) -> &'o NativeComponent<'a> {
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
    pub fn children(&self) -> NativeChildren<'o, 'a> {
        NativeChildren::element(self.component, self.element)
    }
    #[must_use]
    pub fn attributes(&self) -> NativeAttributes<'o, 'a> {
        NativeAttributes {
            component: self.component,
            element: self.element,
            attributes: self.element.open.attrs.iter().enumerate(),
        }
    }
}

/// Actual opening-header iteration with a short original-owner borrow.
pub struct NativeAttributes<'o, 'a> {
    component: &'o NativeComponent<'a>,
    element: &'o Element<'a>,
    attributes: core::iter::Enumerate<core::slice::Iter<'o, Attribute<'a>>>,
}
impl<'o, 'a> Iterator for NativeAttributes<'o, 'a> {
    type Item = NativeAttribute<'o, 'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let (ordinal, attribute) = self.attributes.next()?;
        Some(NativeAttribute {
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
impl ExactSizeIterator for NativeAttributes<'_, '_> {}

/// Full immutable original Attribute, not an arbitrary same-span carrier.
pub struct NativeAttribute<'o, 'a> {
    component: &'o NativeComponent<'a>,
    element: &'o Element<'a>,
    ordinal: usize,
    attribute: &'o Attribute<'a>,
}
impl<'o, 'a> NativeAttribute<'o, 'a> {
    #[must_use]
    pub fn component(&self) -> &'o NativeComponent<'a> {
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
