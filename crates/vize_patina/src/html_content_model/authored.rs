//! Which parse the checker reads (Davinci P4-11a).
//!
//! The checker needs the template **as authored**. The linter's own
//! (`Standard`) parse repairs tree construction in a few situations; where it
//! repaired nothing, its tree already is the authored one and parsing the
//! template a second time is pure cost on the linter's hot path.
//!
//! A repair always leaves a trace in the node locations, which cover each
//! element's start tag: an implicitly closed element is not followed (after
//! its last child) by its own end tag, an inserted `tbody`/`tr` has no source
//! text of its own, and a foster-parented or reconstructed node sits out of
//! source order. [`built_as_authored`] accepts a tree only when none of those
//! traces is present; anything else is re-read with the non-repairing
//! `Quirks` syntax.

use vize_relief::{RootNode, TemplateChildNode};
use vize_s0::Allocator;

use super::build::{authored_skeleton, skeleton};
use super::skeleton::Skeleton;
use crate::ir::TemplateSyntax;
use crate::markup::MarkupDocument;

/// The skeleton of the template `source` as authored, reusing the linter's
/// parse `root` of the same source when that parse repaired nothing.
pub fn template_skeleton<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    root: &'a RootNode<'a>,
) -> Skeleton {
    if built_as_authored(root, source) {
        skeleton(&MarkupDocument::new(root, TemplateSyntax::Vue))
    } else {
        authored_skeleton(allocator, source)
    }
}

/// Whether `root` (a parse of `source`) is the tree as authored: nodes in
/// source order with nothing but whitespace between them (a dropped start
/// tag leaves source no node covers), and every element that is neither void
/// nor self-closing followed, after its last child, by its own end tag.
pub fn built_as_authored(root: &RootNode<'_>, source: &str) -> bool {
    children_end(&root.children, 0, source)
        .is_some_and(|end| blank(source.as_bytes(), end as usize, source.len()))
}

fn blank(bytes: &[u8], from: usize, to: usize) -> bool {
    bytes
        .get(from..to)
        .is_some_and(|gap| gap.iter().all(u8::is_ascii_whitespace))
}

/// The offset after the last of `children` (which start at or after
/// `start`), or `None` when they are not laid out as authored.
fn children_end(children: &[TemplateChildNode<'_>], start: u32, source: &str) -> Option<u32> {
    let mut cursor = start;
    for child in children {
        let span = child.loc().span;
        if span.start < cursor
            || span.start >= span.end
            || !blank(source.as_bytes(), cursor as usize, span.start as usize)
        {
            return None;
        }
        cursor = match child {
            TemplateChildNode::Element(element) => {
                let bytes = source.as_bytes();
                let tag = element.tag.as_bytes();
                let start_tag = bytes.get(span.start as usize..span.end as usize)?;
                if start_tag.first() != Some(&b'<')
                    || !start_tag.get(1..=tag.len())?.eq_ignore_ascii_case(tag)
                {
                    return None;
                }
                if element.is_self_closing || vize_s0::is_void_tag(element.tag) {
                    if !element.children.is_empty() {
                        return None;
                    }
                    span.end
                } else {
                    let content_end = children_end(&element.children, span.end, source)?;
                    end_tag_end(bytes, content_end, tag)?
                }
            }
            TemplateChildNode::Text(_)
            | TemplateChildNode::Comment(_)
            | TemplateChildNode::Interpolation(_) => span.end,
            // Transform-only nodes never come out of the parser.
            _ => return None,
        };
    }
    Some(cursor)
}

/// The offset after `</tag>` when it follows `from` (after whitespace).
fn end_tag_end(bytes: &[u8], from: u32, tag: &[u8]) -> Option<u32> {
    let skip = |at: usize| {
        at + bytes.get(at..).map_or(0, |rest| {
            rest.iter()
                .take_while(|byte| byte.is_ascii_whitespace())
                .count()
        })
    };
    let at = skip(from as usize);
    let name = at + 2;
    if bytes.get(at..name)? != b"</"
        || !bytes.get(name..name + tag.len())?.eq_ignore_ascii_case(tag)
    {
        return None;
    }
    let close = skip(name + tag.len());
    (bytes.get(close) == Some(&b'>')).then(|| u32::try_from(close + 1).ok())?
}
