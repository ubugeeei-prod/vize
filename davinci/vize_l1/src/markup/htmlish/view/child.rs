use super::{ElementView, ParsedComponent};
use crate::markup::{Element, SurfaceChild};

#[derive(Clone, Copy)]
pub(super) enum Parent<'o, 'a> {
    Root,
    Element(&'o Element<'a>),
}

/// Original direct-child iteration. No arbitrary membership constructor.
pub struct ChildrenView<'o, 'a> {
    component: &'o ParsedComponent<'a>,
    parent: Parent<'o, 'a>,
    children: core::iter::Enumerate<core::slice::Iter<'o, SurfaceChild<'a>>>,
}

impl<'o, 'a> ChildrenView<'o, 'a> {
    pub(super) fn root(component: &'o ParsedComponent<'a>) -> Self {
        Self {
            component,
            parent: Parent::Root,
            children: component.component.tree.children.iter().enumerate(),
        }
    }
    pub(super) fn element(component: &'o ParsedComponent<'a>, element: &'o Element<'a>) -> Self {
        Self {
            component,
            parent: Parent::Element(element),
            children: element.children.iter().enumerate(),
        }
    }
}

impl<'o, 'a> Iterator for ChildrenView<'o, 'a> {
    type Item = ChildView<'o, 'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let (ordinal, child) = self.children.next()?;
        Some(ChildView {
            component: self.component,
            parent: self.parent,
            ordinal,
            child,
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.children.size_hint()
    }
}
impl ExactSizeIterator for ChildrenView<'_, '_> {}

/// Actual child, direct parent and ordinal; not a whole-body certificate.
pub struct ChildView<'o, 'a> {
    component: &'o ParsedComponent<'a>,
    parent: Parent<'o, 'a>,
    ordinal: usize,
    child: &'o SurfaceChild<'a>,
}
impl<'o, 'a> ChildView<'o, 'a> {
    /// Borrow this exact original child before its consuming body operation.
    /// No new iteration, parent selection or membership construction occurs.
    #[must_use]
    pub fn reborrow(&self) -> ChildView<'_, 'a> {
        ChildView {
            component: self.component,
            parent: self.parent,
            ordinal: self.ordinal,
            child: self.child,
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
    pub fn surface(&self) -> &'o SurfaceChild<'a> {
        self.child
    }
    /// Consume the child projection without losing original element membership.
    #[must_use]
    pub fn into_element(self) -> Option<ElementView<'o, 'a>> {
        let SurfaceChild::Element(element) = self.child else {
            return None;
        };
        Some(ElementView::from_child(
            self.component,
            self.parent,
            self.ordinal,
            element,
        ))
    }
}
