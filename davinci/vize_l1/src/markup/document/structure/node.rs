//! Body-subtree node views derived from the original private Document callbacks.

use vize_l0::Span;

use super::{DocumentHtmlElement, DocumentHtmlRefusal as Refusal, DocumentHtmlStructure, Element};
use crate::markup::document::{DocumentToken, DocumentTokenKind as Kind, Event, NativeDocument};

mod scalar;

/// A direct original child of a supported HTML body-subtree element.
#[derive(Debug)]
pub enum DocumentHtmlNode<'s, 'o, 'a> {
    Element(DocumentHtmlElement<'s, 'o, 'a>),
    Text(DocumentHtmlText<'s, 'o, 'a>),
    Comment(DocumentHtmlComment<'s, 'o, 'a>),
}

/// A sealed original child cursor. It follows existing element adjacency and
/// skips each child's recorded callback range; it never parses that child again.
/// Scalar readback is linear in the original callback/literal content visited.
///
/// ```compile_fail
/// use vize_l1::markup::document::DocumentHtmlChildNodes;
/// fn forge<'s, 'o, 'a>(original: DocumentHtmlChildNodes<'s, 'o, 'a>) -> DocumentHtmlChildNodes<'s, 'o, 'a> {
///     DocumentHtmlChildNodes { ..original }
/// }
/// ```
#[derive(Debug)]
pub struct DocumentHtmlChildNodes<'s, 'o, 'a> {
    tree: &'s DocumentHtmlStructure<'o, 'a>,
    parent: &'s Element,
    cursor: usize,
    next_element: Option<usize>,
}

pub(super) fn children<'s, 'o, 'a>(
    tree: &'s DocumentHtmlStructure<'o, 'a>,
    parent: &'s Element,
) -> Result<DocumentHtmlChildNodes<'s, 'o, 'a>, Refusal> {
    if !matches!(
        parent.name,
        "body" | "div" | "span" | "br" | "hr" | "img" | "input"
    ) {
        return Err(Refusal::UnsupportedNodeParent(parent.opening));
    }
    if parent.name == "body"
        && let Some(span) = tree.body_tail_text
    {
        return Err(Refusal::OutsideBodyText(span));
    }
    Ok(DocumentHtmlChildNodes {
        tree,
        parent,
        cursor: parent.attributes.end + 1,
        next_element: parent.first_child,
    })
}

impl<'s, 'o, 'a> DocumentHtmlChildNodes<'s, 'o, 'a> {
    fn read(&mut self) -> Result<Option<DocumentHtmlNode<'s, 'o, 'a>>, Refusal> {
        let owner = self.tree.owner;
        while self.cursor < self.parent.content_end {
            let event = owner
                .events
                .get(self.cursor)
                .ok_or(Refusal::InvalidFrame(self.parent.opening))?;
            match event.kind() {
                Kind::OpenTagName => {
                    let index = self.next_element.ok_or(Refusal::InvalidFrame(event.span))?;
                    let element = self
                        .tree
                        .elements
                        .get(index)
                        .ok_or(Refusal::InvalidFrame(event.span))?;
                    if element.authored_name != event.span {
                        return Err(Refusal::InvalidFrame(event.span));
                    }
                    self.next_element = element.next_sibling;
                    self.cursor = if element.closing.is_some() {
                        element.content_end + 1
                    } else {
                        element.content_end
                    };
                    return Ok(Some(DocumentHtmlNode::Element(DocumentHtmlElement {
                        tree: self.tree,
                        element,
                    })));
                }
                Kind::Comment => {
                    self.cursor += 1;
                    let start = event
                        .span
                        .start
                        .checked_sub(4)
                        .ok_or(Refusal::InvalidFrame(event.span))?;
                    let end = event
                        .span
                        .end
                        .checked_add(3)
                        .ok_or(Refusal::InvalidFrame(event.span))?;
                    return Ok(Some(DocumentHtmlNode::Comment(DocumentHtmlComment {
                        tree: self.tree,
                        parent: self.parent,
                        event,
                        frame: Span::new(start, end),
                    })));
                }
                Kind::Text | Kind::TextEntity => {
                    let start = self.cursor;
                    let mut end = event.span.end;
                    let mut nonempty = false;
                    while self.cursor < self.parent.content_end {
                        let part = owner
                            .events
                            .get(self.cursor)
                            .ok_or(Refusal::InvalidFrame(event.span))?;
                        match part.kind() {
                            Kind::Text => {
                                let raw = owner
                                    .source()
                                    .get(part.span.start as usize..part.span.end as usize)
                                    .ok_or(Refusal::InvalidFrame(part.span))?;
                                nonempty |= raw.chars().any(|ch| ch != '\0');
                            }
                            Kind::TextEntity => nonempty = true,
                            _ => break,
                        }
                        end = part.span.end;
                        self.cursor += 1;
                    }
                    // The browser ignores literal NULL character tokens, so an
                    // entirely NULL run does not mint an empty DOM text node.
                    if nonempty {
                        return Ok(Some(DocumentHtmlNode::Text(DocumentHtmlText {
                            tree: self.tree,
                            parent: self.parent,
                            parts: start..self.cursor,
                            span: Span::new(event.span.start, end),
                        })));
                    }
                }
                _ => return Err(Refusal::InvalidFrame(event.span)),
            }
        }
        if self.next_element.is_some() {
            return Err(Refusal::InvalidFrame(self.parent.opening));
        }
        Ok(None)
    }
}

impl<'s, 'o, 'a> Iterator for DocumentHtmlChildNodes<'s, 'o, 'a> {
    type Item = Result<DocumentHtmlNode<'s, 'o, 'a>, Refusal>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read() {
            Ok(node) => node.map(Ok),
            Err(refusal) => {
                self.cursor = self.parent.content_end;
                self.next_element = None;
                Some(Err(refusal))
            }
        }
    }
}

/// One original direct text node, retaining every literal/reference callback.
/// No decoded text buffer is constructed. Literal CRLF/CR preprocesses to LF
/// and literal NULL is ignored; original reference scalars bypass preprocessing.
///
/// ```compile_fail
/// use vize_l1::markup::document::DocumentHtmlText;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<DocumentHtmlText<'static, 'static, 'static>>();
/// ```
///
/// ```
/// use vize_l0::{Allocator, SourceRoot};
/// use vize_l1::markup::document::{DocumentHtmlNode, NativeDocument};
/// let arena = Allocator::default();
/// let source = "<!DOCTYPE html><html><head></head><body>&amp;</body></html>";
/// let owner = NativeDocument::lex_in(&arena, SourceRoot::new(source).unwrap());
/// let tree = owner.html_structure().unwrap();
/// let body = tree.elements().nth(2).unwrap();
/// let DocumentHtmlNode::Text(text) = body.child_nodes().unwrap().next().unwrap().unwrap() else { panic!() };
/// assert_eq!(text.authored(), "&amp;");
/// assert!(text.characters().eq("&".chars()));
/// assert!(core::ptr::eq(text.owner(), &owner));
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::document::DocumentHtmlText;
/// fn forge<'s, 'o, 'a>(original: DocumentHtmlText<'s, 'o, 'a>) -> DocumentHtmlText<'s, 'o, 'a> {
///     DocumentHtmlText { ..original }
/// }
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::document::{DocumentHtmlNode, DocumentHtmlStructure, DocumentHtmlText};
/// fn outlive_tree<'o, 'a>(tree: DocumentHtmlStructure<'o, 'a>) -> DocumentHtmlText<'o, 'o, 'a> {
///     let body = tree.elements().nth(2).unwrap();
///     let DocumentHtmlNode::Text(text) = body.child_nodes().unwrap().next().unwrap().unwrap() else { panic!() };
///     text
/// }
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::document::{DocumentHtmlNode, NativeDocument};
/// fn drop_owner(owner: NativeDocument<'_>) {
///     let tree = owner.html_structure().unwrap();
///     let body = tree.elements().nth(2).unwrap();
///     let DocumentHtmlNode::Text(text) = body.child_nodes().unwrap().next().unwrap().unwrap() else { panic!() };
///     drop(owner);
///     assert!(text.characters().next().is_some());
/// }
/// ```
#[derive(Debug)]
pub struct DocumentHtmlText<'s, 'o, 'a> {
    tree: &'s DocumentHtmlStructure<'o, 'a>,
    parent: &'s Element,
    parts: core::ops::Range<usize>,
    span: Span,
}

impl<'s, 'o, 'a> DocumentHtmlText<'s, 'o, 'a> {
    #[must_use]
    pub fn owner(&self) -> &'o NativeDocument<'a> {
        self.tree.owner
    }
    #[must_use]
    pub fn parent(&self) -> DocumentHtmlElement<'s, 'o, 'a> {
        DocumentHtmlElement {
            tree: self.tree,
            element: self.parent,
        }
    }
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }
    #[must_use]
    pub fn authored(&self) -> &'a str {
        self.span.slice(self.owner().source())
    }

    /// Actual original callback views; decoded payload identity stays retained.
    pub fn tokens(&self) -> impl Iterator<Item = DocumentToken<'o, 'a>> + 'o {
        let owner = self.tree.owner;
        owner
            .events
            .get(self.parts.clone())
            .into_iter()
            .flatten()
            .map(move |event| DocumentToken { owner, event })
    }

    pub fn characters(&self) -> impl Iterator<Item = char> + 'o {
        let owner = self.tree.owner;
        owner
            .events
            .get(self.parts.clone())
            .into_iter()
            .flatten()
            .flat_map(move |event| scalar::text(event, owner.source()))
    }
}

/// One original ordinary comment. Character references remain literal data.
/// Abrupt/nested/alternative comment frames already refuse HTML construction.
///
/// ```compile_fail
/// use vize_l1::markup::document::DocumentHtmlComment;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<DocumentHtmlComment<'static, 'static, 'static>>();
/// ```
///
/// ```compile_fail
/// use vize_l1::markup::document::DocumentHtmlComment;
/// fn forge<'s, 'o, 'a>(original: DocumentHtmlComment<'s, 'o, 'a>) -> DocumentHtmlComment<'s, 'o, 'a> {
///     DocumentHtmlComment { ..original }
/// }
/// ```
#[derive(Debug)]
pub struct DocumentHtmlComment<'s, 'o, 'a> {
    tree: &'s DocumentHtmlStructure<'o, 'a>,
    parent: &'s Element,
    event: &'o Event,
    frame: Span,
}

impl<'s, 'o, 'a> DocumentHtmlComment<'s, 'o, 'a> {
    #[must_use]
    pub fn owner(&self) -> &'o NativeDocument<'a> {
        self.tree.owner
    }
    #[must_use]
    pub fn parent(&self) -> DocumentHtmlElement<'s, 'o, 'a> {
        DocumentHtmlElement {
            tree: self.tree,
            element: self.parent,
        }
    }
    #[must_use]
    pub fn span(&self) -> Span {
        self.frame
    }
    #[must_use]
    pub fn content_span(&self) -> Span {
        self.event.span
    }
    #[must_use]
    pub fn authored_content(&self) -> &'a str {
        self.event.span.slice(self.owner().source())
    }
    #[must_use]
    pub fn token(&self) -> DocumentToken<'o, 'a> {
        DocumentToken {
            owner: self.tree.owner,
            event: self.event,
        }
    }
    pub fn characters(&self) -> impl Iterator<Item = char> + 'a {
        scalar::comment(self.authored_content())
    }
}

#[cfg(test)]
mod tests;
