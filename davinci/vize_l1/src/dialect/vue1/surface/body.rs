//! Sealed direct-child membership, joined to original callback observations.

use super::super::text::{TextBinding, TextBoundaryKind};
use super::ComponentParse;
use crate::embed::syntax::EmbedHole;
use crate::{Element, ElementClose, SurfaceChild};
use oxc_ast::ast::Expression;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRefusal {
    ForeignComponent,
    NotInterpolation,
    UnmatchedCallback,
    RecoveredComponent,
    Verbatim,
    Boundary(TextBoundaryKind),
    NativeHole(EmbedHole),
}

/// Only the owner's actual original children can produce these views.
pub struct TextChildren<'o, 'a> {
    component: &'o ComponentParse<'a>,
    parent: Option<&'o Element<'a>>,
    recovered: bool,
    children: core::iter::Enumerate<core::slice::Iter<'o, SurfaceChild<'a>>>,
}
impl<'o, 'a> Iterator for TextChildren<'o, 'a> {
    type Item = TextChild<'o, 'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let (ordinal, surface) = self.children.next()?;
        Some(TextChild {
            component: self.component,
            parent: self.parent,
            recovered: self.recovered,
            ordinal,
            surface,
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.children.size_hint()
    }
}
impl ExactSizeIterator for TextChildren<'_, '_> {}

/// An original direct parent and ordinal, never a caller-selected raw node.
///
/// ```compile_fail
/// use vize_l1::{SurfaceChild, dialect::vue1::surface::TextChild};
/// fn forge<'o, 'a>(raw: &'o SurfaceChild<'a>) -> TextChild<'o, 'a> { raw.into() }
/// ```
#[derive(Debug)]
pub struct TextChild<'o, 'a> {
    component: &'o ComponentParse<'a>,
    parent: Option<&'o Element<'a>>,
    recovered: bool,
    ordinal: usize,
    surface: &'o SurfaceChild<'a>,
}
impl<'o, 'a> TextChild<'o, 'a> {
    pub fn reborrow(&self) -> TextChild<'_, 'a> {
        TextChild {
            component: self.component,
            parent: self.parent,
            recovered: self.recovered,
            ordinal: self.ordinal,
            surface: self.surface,
        }
    }
    pub fn component(&self) -> &'o ComponentParse<'a> {
        self.component
    }
    pub fn parent_element(&self) -> Option<&'o Element<'a>> {
        self.parent
    }
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub fn surface(&self) -> &'o SurfaceChild<'a> {
        self.surface
    }
    /// Traverse only this original element's actual children. Recovery state
    /// follows this same traversal, without a separate header or body walk.
    pub fn children(&self) -> Option<TextChildren<'o, 'a>> {
        let SurfaceChild::Element(element) = self.surface else {
            return None;
        };
        Some(TextChildren {
            component: self.component,
            parent: Some(element),
            recovered: self.recovered || recovered(element),
            children: element.children.iter().enumerate(),
        })
    }
}

impl<'a> ComponentParse<'a> {
    pub fn children(&self) -> TextChildren<'_, 'a> {
        TextChildren {
            component: self,
            parent: None,
            recovered: false,
            children: self.tree.children.iter().enumerate(),
        }
    }
    pub fn authored_children(&self) -> Option<TextChildren<'_, 'a>> {
        Some(TextChildren {
            component: self,
            parent: None,
            recovered: false,
            children: self.authored.as_ref()?.children.iter().enumerate(),
        })
    }
    /// Original callback/CST join. This performs no decoding or parsing.
    pub fn text_for<'o>(
        &'o self,
        child: TextChild<'o, 'a>,
    ) -> Result<TextView<'o, 'a>, TextRefusal> {
        if !core::ptr::eq(self, child.component) {
            return Err(TextRefusal::ForeignComponent);
        }
        let SurfaceChild::Interpolation(node) = child.surface else {
            return Err(TextRefusal::NotInterpolation);
        };
        if !self.errors.is_empty() || child.recovered {
            return Err(TextRefusal::RecoveredComponent);
        }
        if child.parent.is_some_and(|parent| parent.open.is_verbatim()) {
            return Err(TextRefusal::Verbatim);
        }
        if let Some(boundary) = self.text_boundaries.first() {
            return Err(TextRefusal::Boundary(boundary.kind));
        }
        let start = self
            .block
            .offset_of(node.content.text)
            .ok_or(TextRefusal::UnmatchedCallback)?;
        let index = self
            .bindings
            .binary_search_by_key(&start, TextBinding::cst_start)
            .map_err(|_| TextRefusal::UnmatchedCallback)?;
        let binding = self
            .bindings
            .get(index)
            .ok_or(TextRefusal::UnmatchedCallback)?;
        if let Some(kind) = binding.boundary() {
            return Err(TextRefusal::Boundary(kind));
        }
        if node.is_raw_html()
            || node.open.is_missing()
            || node.content.is_missing()
            || node.close.is_missing()
            || node.open.text != "{{"
            || node.close.text != "}}"
            || !binding.matches(self.block, node.content.text)
            || self.block.span_of(node.open.text).map(|span| span.start)
                != Some(binding.span().start)
            || self.block.span_of(node.close.text).map(|span| span.end) != Some(binding.span().end)
        {
            return Err(TextRefusal::UnmatchedCallback);
        }
        let syntax = binding.syntax().ok_or(TextRefusal::UnmatchedCallback)?;
        if let Some(hole) = syntax.hole() {
            return Err(TextRefusal::NativeHole(hole));
        }
        let expression = syntax.expression().ok_or(TextRefusal::UnmatchedCallback)?;
        Ok(TextView {
            binding,
            child,
            expression,
        })
    }
}

/// Borrowed original callback, expression and CST occurrence custody.
///
/// ```compile_fail
/// use vize_l0::Allocator;
/// use vize_l1::dialect::vue1::surface;
/// let arena = Allocator::default();
/// let view = {
///     let owner = surface::parse_component(&arena, "{{ x }}").unwrap();
///     owner.text_for(owner.children().next().unwrap()).unwrap()
/// };
/// println!("{:?}", view.expression());
/// ```
#[derive(Debug)]
pub struct TextView<'o, 'a> {
    binding: &'o TextBinding<'a>,
    child: TextChild<'o, 'a>,
    expression: &'o Expression<'a>,
}
impl<'o, 'a> TextView<'o, 'a> {
    pub fn binding(&self) -> &'o TextBinding<'a> {
        self.binding
    }
    pub fn child(&self) -> &TextChild<'o, 'a> {
        &self.child
    }
    pub fn expression(&self) -> &Expression<'a> {
        self.expression
    }
}

fn recovered(element: &Element<'_>) -> bool {
    element.open.lt_name.is_missing()
        || element.open.gt.is_missing()
        || element
            .open
            .slash
            .as_ref()
            .is_some_and(|token| token.is_missing())
        || match &element.close {
            ElementClose::Present(close) => {
                close.lt_slash_name.is_missing() || close.gt.is_missing()
            }
            ElementClose::Missing | ElementClose::Implicit => true,
            ElementClose::NotExpected => false,
        }
}
