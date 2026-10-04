//! Original Vue 2 callback/filter-chain custody at a sealed direct CST child.

use super::super::text::{FilterChain, TextBinding, TextBoundaryKind};
use super::ComponentParse;
use crate::embed::syntax::EmbedHole;
use crate::{Element, ElementClose, SurfaceChild};

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

/// A borrow of actual original children, with no allocation or additional walk.
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

/// Sealed original component, direct parent and ordinal membership.
///
/// ```compile_fail
/// use vize_l1::{SurfaceChild, dialect::vue2::surface::TextChild};
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
    /// Recovery follows this same original visit, never a later ancestor walk.
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
    /// The original source and arena must outlive the component and its views.
    ///
    /// ```compile_fail
    /// use vize_l0::Allocator;
    /// use vize_l1::dialect::vue2::surface;
    /// let arena = Allocator::default();
    /// let owner = {
    ///     let source = String::from("{{ x }}");
    ///     surface::parse_component(&arena, &source).unwrap()
    /// };
    /// println!("{:?}", owner.tree());
    /// ```
    /// ```compile_fail
    /// use vize_l0::Allocator;
    /// use vize_l1::dialect::vue2::surface;
    /// let owner = {
    ///     let arena = Allocator::default();
    ///     surface::parse_component(&arena, "{{ x }}").unwrap()
    /// };
    /// println!("{:?}", owner.tree());
    /// ```
    pub fn children(&self) -> TextChildren<'_, 'a> {
        TextChildren {
            component: self,
            parent: None,
            recovered: false,
            children: self.tree.children.iter().enumerate(),
        }
    }
    /// Only a genuine existing repair can supply this alternative membership.
    /// Its children stay inspectable but cannot admit original body custody.
    pub fn authored_children(&self) -> Option<TextChildren<'_, 'a>> {
        Some(TextChildren {
            component: self,
            parent: None,
            recovered: true,
            children: self.authored.as_ref()?.children.iter().enumerate(),
        })
    }
    /// Join the actual CST child to its existing monotonic callback observation.
    /// Binary search is O(log n); no resident index, decoding or parsing occurs.
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
        if let Some(boundary) = self.unsupported.first() {
            return Err(TextRefusal::Boundary(boundary.kind));
        }
        let start = self
            .block
            .offset_of(node.open.text)
            .ok_or(TextRefusal::UnmatchedCallback)?;
        let index = self
            .bindings
            .binary_search_by_key(&start, |binding| binding.span().start)
            .map_err(|_| TextRefusal::UnmatchedCallback)?;
        let binding = self
            .bindings
            .get(index)
            .ok_or(TextRefusal::UnmatchedCallback)?;
        if let Some(boundary) = binding.boundaries().first() {
            return Err(TextRefusal::Boundary(boundary.kind));
        }
        let open = self.block.span_of(node.open.text);
        let content = self.block.span_of(node.content.text);
        let close = self.block.span_of(node.close.text);
        if node.is_raw_html()
            || node.open.is_missing()
            || node.content.is_missing()
            || node.close.is_missing()
            || node.open.text != "{{"
            || node.close.text != "}}"
            || !binding.matches(self.block, node.content.text)
            || open.map(|span| span.start) != Some(binding.span().start)
            || close.map(|span| span.end) != Some(binding.span().end)
            || open.map(|span| span.end) != content.map(|span| span.start)
            || content.map(|span| span.end) != close.map(|span| span.start)
        {
            return Err(TextRefusal::UnmatchedCallback);
        }
        let chain = if let Some(chain) = binding.admitted() {
            chain
        } else {
            let chain = binding.chain().ok_or(TextRefusal::UnmatchedCallback)?;
            let hole = chain.base().hole().or_else(|| {
                chain
                    .filters()
                    .iter()
                    .flat_map(|filter| filter.arguments())
                    .find_map(|arg| arg.hole())
            });
            return Err(hole.map_or(TextRefusal::UnmatchedCallback, TextRefusal::NativeHole));
        };
        Ok(TextView {
            binding,
            child,
            chain,
        })
    }
}

/// Borrowed original chain/CST custody; registry/runtime/File meaning is absent.
///
/// ```compile_fail
/// use vize_l0::Allocator;
/// use vize_l1::dialect::vue2::surface;
/// let arena = Allocator::default();
/// let view = {
///     let owner = surface::parse_component(&arena, "{{ x | upper }}").unwrap();
///     owner.text_for(owner.children().next().unwrap()).unwrap()
/// };
/// println!("{:?}", view.chain());
/// ```
/// ```compile_fail
/// use vize_l0::Allocator;
/// use vize_l1::dialect::vue2::surface;
/// let arena = Allocator::default();
/// let owner = surface::parse_component(&arena, "{{ x }}").unwrap();
/// let view = owner.text_for(owner.children().next().unwrap()).unwrap();
/// let moved = owner;
/// println!("{:?}", view.chain());
/// ```
/// ```compile_fail
/// use vize_l1::dialect::vue2::{surface::{TextChild, TextView}, text::{FilterChain, TextBinding}};
/// fn forge<'o, 'a>(binding: &'o TextBinding<'a>, child: TextChild<'o, 'a>, chain: &'o FilterChain<'a>) -> TextView<'o, 'a> {
///     TextView { binding, child, chain }
/// }
/// ```
#[derive(Debug)]
pub struct TextView<'o, 'a> {
    binding: &'o TextBinding<'a>,
    child: TextChild<'o, 'a>,
    chain: &'o FilterChain<'a>,
}
impl<'o, 'a> TextView<'o, 'a> {
    pub fn binding(&self) -> &'o TextBinding<'a> {
        self.binding
    }
    pub fn child(&self) -> &TextChild<'o, 'a> {
        &self.child
    }
    pub fn chain(&self) -> &'o FilterChain<'a> {
        self.chain
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
