//! The page ↔ tree bridge: mirroring a live S1 tree into a page, and
//! rebuilding the tree from a page that tiles its source.

use vize_s0::{Allocator, Box, Vec as ArenaVec};
use vize_s1::{
    AttrValue, Attribute, CloseTag, Element, ElementClose, Interpolation, OpenTag, SurfaceChild,
    SurfaceTree, Token, TokenStatus,
};

use super::{
    PageAttrValue, PageAttribute, PageClose, PageElement, PageInterpolation, PageNode, PageToken,
    SurfacePage, TileError,
};

impl SurfacePage {
    /// Mirror a live S1 tree. Offsets are relative to `tree.source`.
    #[must_use]
    pub fn of(tree: &SurfaceTree<'_>) -> Self {
        let base = tree.source.as_ptr() as usize;
        Self {
            children: tree
                .children
                .iter()
                .map(|child| node(base, child))
                .collect(),
        }
    }

    /// Rebuild the S1 tree over `source`, after [`Self::check_tiles`].
    ///
    /// # Errors
    ///
    /// The [`TileError`] when the page does not tile `source`.
    pub fn materialize<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
    ) -> Result<SurfaceTree<'a>, TileError> {
        self.check_tiles(source)?;
        let build = Build { allocator, source };
        Ok(SurfaceTree {
            source,
            children: build.children(&self.children),
        })
    }
}

fn offset(base: usize, slice: &str) -> u32 {
    (slice.as_ptr() as usize - base) as u32
}

fn token(base: usize, token: &Token<'_>) -> PageToken {
    // `leading` ends where `text` starts (the tiling law), and an empty
    // `leading` need not point into the source at all, so its start is
    // measured back from `text` rather than from its own pointer.
    let text = offset(base, token.text);
    PageToken {
        start: text - token.leading.len() as u32,
        text,
        end: text + token.text.len() as u32,
        missing: token.status == TokenStatus::Missing,
    }
}

fn node(base: usize, child: &SurfaceChild<'_>) -> PageNode {
    match child {
        SurfaceChild::Element(element) => PageNode::Element(PageElement {
            lt_name: token(base, &element.open.lt_name),
            attrs: element
                .open
                .attrs
                .iter()
                .map(|attr| attribute(base, attr))
                .collect(),
            slash: element.open.slash.as_ref().map(|slash| token(base, slash)),
            gt: token(base, &element.open.gt),
            children: element
                .children
                .iter()
                .map(|child| node(base, child))
                .collect(),
            close: match &element.close {
                ElementClose::Present(close) => PageClose::Tag {
                    lt_slash_name: token(base, &close.lt_slash_name),
                    gt: token(base, &close.gt),
                },
                ElementClose::Implicit => PageClose::Implicit,
                ElementClose::Missing => PageClose::Missing,
                ElementClose::NotExpected => PageClose::NotExpected,
            },
        }),
        SurfaceChild::Interpolation(node) => PageNode::Interpolation(PageInterpolation {
            open: token(base, &node.open),
            content: token(base, &node.content),
            close: token(base, &node.close),
        }),
        SurfaceChild::Text(t) => PageNode::Text(token(base, t)),
        SurfaceChild::Comment(t) => PageNode::Comment(token(base, t)),
        SurfaceChild::Cdata(t) => PageNode::Cdata(token(base, t)),
        SurfaceChild::ProcessingInstruction(t) => PageNode::ProcessingInstruction(token(base, t)),
        SurfaceChild::Unexpected(t) => PageNode::Unexpected(token(base, t)),
    }
}

fn attribute(base: usize, attr: &Attribute<'_>) -> PageAttribute {
    PageAttribute {
        name: token(base, &attr.name),
        eq: attr.eq.as_ref().map(|eq| token(base, eq)),
        value: attr.value.as_ref().map(|value| PageAttrValue {
            open_quote: value.open_quote.as_ref().map(|quote| token(base, quote)),
            content: token(base, &value.content),
            close_quote: value.close_quote.as_ref().map(|quote| token(base, quote)),
        }),
    }
}

struct Build<'a> {
    allocator: &'a Allocator,
    source: &'a str,
}

impl<'a> Build<'a> {
    fn token(&self, token: &PageToken) -> Token<'a> {
        // `materialize` checked that the page tiles `source`, so every range
        // is in bounds on a character boundary; an empty slice is unreachable.
        let slice = |from: u32, to: u32| {
            self.source
                .get(from as usize..to as usize)
                .unwrap_or_default()
        };
        Token {
            leading: slice(token.start, token.text),
            text: slice(token.text, token.end),
            status: if token.missing {
                TokenStatus::Missing
            } else {
                TokenStatus::Present
            },
        }
    }

    fn opt(&self, token: Option<&PageToken>) -> Option<Token<'a>> {
        token.map(|token| self.token(token))
    }

    fn children(&self, nodes: &[PageNode]) -> ArenaVec<'a, SurfaceChild<'a>> {
        let mut out = ArenaVec::new_in(&self.allocator);
        for node in nodes {
            out.push(match node {
                PageNode::Element(element) => {
                    SurfaceChild::Element(Box::new_in(self.element(element), &self.allocator))
                }
                PageNode::Interpolation(node) => SurfaceChild::Interpolation(Box::new_in(
                    Interpolation {
                        open: self.token(&node.open),
                        content: self.token(&node.content),
                        close: self.token(&node.close),
                    },
                    &self.allocator,
                )),
                PageNode::Text(t) => SurfaceChild::Text(self.token(t)),
                PageNode::Comment(t) => SurfaceChild::Comment(self.token(t)),
                PageNode::Cdata(t) => SurfaceChild::Cdata(self.token(t)),
                PageNode::ProcessingInstruction(t) => {
                    SurfaceChild::ProcessingInstruction(self.token(t))
                }
                PageNode::Unexpected(t) => SurfaceChild::Unexpected(self.token(t)),
            });
        }
        out
    }

    fn element(&self, element: &PageElement) -> Element<'a> {
        let mut attrs = ArenaVec::new_in(&self.allocator);
        for attr in &element.attrs {
            attrs.push(Attribute {
                name: self.token(&attr.name),
                eq: self.opt(attr.eq.as_ref()),
                value: attr.value.as_ref().map(|value| AttrValue {
                    open_quote: self.opt(value.open_quote.as_ref()),
                    content: self.token(&value.content),
                    close_quote: self.opt(value.close_quote.as_ref()),
                }),
            });
        }
        Element {
            open: OpenTag {
                lt_name: self.token(&element.lt_name),
                attrs,
                slash: self.opt(element.slash.as_ref()),
                gt: self.token(&element.gt),
            },
            children: self.children(&element.children),
            close: match &element.close {
                PageClose::Tag { lt_slash_name, gt } => ElementClose::Present(CloseTag {
                    lt_slash_name: self.token(lt_slash_name),
                    gt: self.token(gt),
                }),
                PageClose::Implicit => ElementClose::Implicit,
                PageClose::Missing => ElementClose::Missing,
                PageClose::NotExpected => ElementClose::NotExpected,
            },
        }
    }
}
