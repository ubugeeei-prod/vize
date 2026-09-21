//! The S1-subtree deletion vocabulary (P3-14): every node and attribute of
//! the artifact's lossless surface tree, as the byte range whose removal
//! deletes exactly that subtree.
//!
//! S1 renders back to its input byte for byte (TS-19) and parses any input
//! (hole policy), so deleting a node's bytes always leaves a re-parseable,
//! re-printable artifact — the property llvm-reduce's IR-aware passes buy
//! with a verifier, bought here by the stage's own contract. A node's range
//! starts at its first token's `leading` slice, so a deleted element takes
//! its indentation with it instead of leaving blank lines behind.
//!
//! The SFC is parsed whole (blocks as markup, script/style as raw text), so
//! `<script>` and `<style>` blocks are candidates too, and ranges are file
//! offsets. Candidates come in pre-order — an element, then its attributes,
//! then its children — so a driver trying them in order tries the largest
//! deletions first.

use std::ops::Range;

use vize_s0::{Allocator, String};
use vize_s1::{Attribute, Element, ElementClose, SurfaceChild, Token};

/// Every deletion candidate of `source`, in pre-order.
pub(crate) fn candidates(source: &str) -> Vec<Range<usize>> {
    let allocator = Allocator::default();
    let (tree, _errors) = vize_s1::parse(&allocator, source);
    let mut out = Vec::new();
    let base = Base(source.as_ptr() as usize);
    children(&tree.children, base, &mut out);
    out
}

/// Offset arithmetic against the parsed source: every token slice points
/// into it (S1 owns no strings).
#[derive(Clone, Copy)]
struct Base(usize);

impl Base {
    fn start(self, slice: &str) -> usize {
        slice.as_ptr() as usize - self.0
    }

    fn end(self, slice: &str) -> usize {
        self.start(slice) + slice.len()
    }

    /// A token's range, its leading slice included.
    fn token(self, token: &Token<'_>) -> Range<usize> {
        self.start(token.leading)..self.end(token.text)
    }
}

fn children(nodes: &[SurfaceChild<'_>], base: Base, out: &mut Vec<Range<usize>>) {
    for node in nodes {
        match node {
            SurfaceChild::Element(element) => {
                out.push(element_range(element, base));
                for attribute in element.open.attrs.iter() {
                    out.push(attribute_range(attribute, base));
                }
                children(&element.children, base, out);
            }
            SurfaceChild::Interpolation(node) => {
                out.push(base.start(node.open.leading)..base.end(node.close.text));
            }
            SurfaceChild::Text(token)
            | SurfaceChild::Comment(token)
            | SurfaceChild::Cdata(token)
            | SurfaceChild::ProcessingInstruction(token)
            | SurfaceChild::Unexpected(token) => out.push(base.token(token)),
        }
    }
}

/// An element's range: its open tag's leading slice to the end of its
/// close tag, or — when it has none — to the end of its last rendered
/// piece.
fn element_range(element: &Element<'_>, base: Base) -> Range<usize> {
    let start = base.start(element.open.lt_name.leading);
    let end = match &element.close {
        ElementClose::Present(close) => base.end(close.gt.text),
        ElementClose::Implicit | ElementClose::Missing | ElementClose::NotExpected => {
            let open_end = base.end(element.open.gt.text);
            let mut last = Vec::new();
            children(&element.children, base, &mut last);
            last.iter()
                .map(|range| range.end)
                .max()
                .map_or(open_end, |end| end.max(open_end))
        }
    };
    start..end
}

fn attribute_range(attribute: &Attribute<'_>, base: Base) -> Range<usize> {
    let start = base.start(attribute.name.leading);
    let mut end = base.end(attribute.name.text);
    if let Some(eq) = &attribute.eq {
        end = end.max(base.end(eq.text));
    }
    if let Some(value) = &attribute.value {
        end = end.max(base.end(value.content.text));
        if let Some(close) = &value.close_quote {
            end = end.max(base.end(close.text));
        }
    }
    start..end
}

/// `source` with every range in `ranges` removed. Ranges may nest or
/// overlap (a chunk can hold an element and its own attributes); their
/// union is what goes.
pub(crate) fn delete(source: &str, ranges: &[Range<usize>]) -> String {
    let mut sorted: Vec<Range<usize>> = ranges.to_vec();
    sorted.sort_by_key(|range| range.start);
    let mut out = String::with_capacity(source.len());
    let mut cursor = 0;
    for range in sorted {
        if range.end <= cursor {
            continue;
        }
        let start = range.start.max(cursor);
        out.push_str(&source[cursor..start]);
        cursor = range.end;
    }
    out.push_str(&source[cursor..]);
    out
}
