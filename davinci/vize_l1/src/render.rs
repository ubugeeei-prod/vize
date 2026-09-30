//! Rendering back to source, the byte-fidelity check (TS-19's oracle),
//! and the typed-hole census the tests pin.
//!
//! The render order is the documented canonical token order: for every
//! token, `leading` then `text`; children in tree order; an element as
//! open tag (`lt_name`, attributes as `name`/`eq`/`open_quote`/`content`/
//! `close_quote`, `slash`, `gt`), then children, then its close tag.
//! `Missing` tokens, `ElementClose::Implicit`, and `ElementClose::Missing`
//! render zero bytes.

use crate::surface::{
    Attribute, CloseTag, Element, ElementClose, Interpolation, SurfaceChild, SurfaceTree, Token,
};

/// Feed every rendered piece to `sink`, in canonical order. Concatenating
/// the pieces reproduces the source bytes exactly (checked by
/// [`check_fidelity`], debug-asserted on every [`crate::parse::parse`]).
pub fn render<'a>(tree: &SurfaceTree<'a>, sink: &mut dyn FnMut(&'a str)) {
    walk_children(&tree.children, sink);
}

fn emit<'a>(token: &Token<'a>, sink: &mut dyn FnMut(&'a str)) {
    sink(token.leading);
    sink(token.text);
}

fn walk_children<'a>(children: &[SurfaceChild<'a>], sink: &mut dyn FnMut(&'a str)) {
    for child in children {
        match child {
            SurfaceChild::Element(element) => walk_element(element, sink),
            SurfaceChild::Interpolation(node) => walk_interpolation(node, sink),
            SurfaceChild::Text(token)
            | SurfaceChild::Comment(token)
            | SurfaceChild::Cdata(token)
            | SurfaceChild::ProcessingInstruction(token)
            | SurfaceChild::Unexpected(token) => emit(token, sink),
        }
    }
}

fn walk_interpolation<'a>(node: &Interpolation<'a>, sink: &mut dyn FnMut(&'a str)) {
    emit(&node.open, sink);
    emit(&node.content, sink);
    emit(&node.close, sink);
}

fn walk_attribute<'a>(attr: &Attribute<'a>, sink: &mut dyn FnMut(&'a str)) {
    emit(&attr.name, sink);
    if let Some(eq) = &attr.eq {
        emit(eq, sink);
    }
    if let Some(value) = &attr.value {
        if let Some(open_quote) = &value.open_quote {
            emit(open_quote, sink);
        }
        emit(&value.content, sink);
        if let Some(close_quote) = &value.close_quote {
            emit(close_quote, sink);
        }
    }
}

fn walk_element<'a>(element: &Element<'a>, sink: &mut dyn FnMut(&'a str)) {
    emit(&element.open.lt_name, sink);
    for attr in &element.open.attrs {
        walk_attribute(attr, sink);
    }
    if let Some(slash) = &element.open.slash {
        emit(slash, sink);
    }
    emit(&element.open.gt, sink);
    walk_children(&element.children, sink);
    if let ElementClose::Present(CloseTag { lt_slash_name, gt }) = &element.close {
        emit(lt_slash_name, sink);
        emit(gt, sink);
    }
}

/// Verify `render(tree) == tree.source` as bytes, without allocating:
/// each rendered piece is compared at its running offset. `Err` carries
/// the byte offset of the first divergence (or the rendered length, when
/// rendering stopped short of the source).
pub fn check_fidelity(tree: &SurfaceTree<'_>) -> Result<(), usize> {
    let src = tree.source.as_bytes();
    let mut pos = 0usize;
    let mut mismatch: Option<usize> = None;
    render(tree, &mut |piece| {
        if mismatch.is_some() {
            return;
        }
        let bytes = piece.as_bytes();
        if src.get(pos..pos + bytes.len()) == Some(bytes) {
            pos += bytes.len();
        } else {
            mismatch = Some(pos);
        }
    });
    match mismatch {
        Some(at) => Err(at),
        None if pos == src.len() => Ok(()),
        None => Err(pos),
    }
}

/// The typed-hole census: how many holes of each policy clause the tree
/// carries. The malformed battery pins these exactly (TS-19's structural
/// half).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HoleCounts {
    /// Clause 1, token level: `Missing` tokens.
    pub missing_tokens: usize,
    /// Clause 1, node level: `ElementClose::Missing`.
    pub missing_close_tags: usize,
    /// Clause 2: `Unexpected` children.
    pub unexpected_nodes: usize,
}

pub fn hole_counts(tree: &SurfaceTree<'_>) -> HoleCounts {
    let mut counts = HoleCounts::default();
    count_children(&tree.children, &mut counts);
    counts
}

fn count_token(token: &Token<'_>, counts: &mut HoleCounts) {
    if token.is_missing() {
        counts.missing_tokens += 1;
    }
}

fn count_children(children: &[SurfaceChild<'_>], counts: &mut HoleCounts) {
    for child in children {
        match child {
            SurfaceChild::Element(element) => count_element(element, counts),
            SurfaceChild::Interpolation(node) => {
                count_token(&node.open, counts);
                count_token(&node.content, counts);
                count_token(&node.close, counts);
            }
            SurfaceChild::Unexpected(token) => {
                counts.unexpected_nodes += 1;
                count_token(token, counts);
            }
            SurfaceChild::Text(token)
            | SurfaceChild::Comment(token)
            | SurfaceChild::Cdata(token)
            | SurfaceChild::ProcessingInstruction(token) => count_token(token, counts),
        }
    }
}

fn count_element(element: &Element<'_>, counts: &mut HoleCounts) {
    count_token(&element.open.lt_name, counts);
    for attr in &element.open.attrs {
        count_token(&attr.name, counts);
        if let Some(eq) = &attr.eq {
            count_token(eq, counts);
        }
        if let Some(value) = &attr.value {
            if let Some(open_quote) = &value.open_quote {
                count_token(open_quote, counts);
            }
            count_token(&value.content, counts);
            if let Some(close_quote) = &value.close_quote {
                count_token(close_quote, counts);
            }
        }
    }
    if let Some(slash) = &element.open.slash {
        count_token(slash, counts);
    }
    count_token(&element.open.gt, counts);
    count_children(&element.children, counts);
    match &element.close {
        ElementClose::Present(CloseTag { lt_slash_name, gt }) => {
            count_token(lt_slash_name, counts);
            count_token(gt, counts);
        }
        ElementClose::Missing => counts.missing_close_tags += 1,
        ElementClose::Implicit | ElementClose::NotExpected => {}
    }
}
