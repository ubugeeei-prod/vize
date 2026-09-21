//! Tiling: a page's tokens must cover the block source exactly, in render
//! order, each starting where the previous one ended.

use core::fmt;

use super::{PageClose, PageNode, PageToken, SurfacePage};

/// Why a page's tokens do not tile a source. `index` is the token's
/// 0-based position in canonical render order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TileError {
    /// A token does not start where the previous one ended.
    Gap {
        index: usize,
        role: &'static str,
        start: u32,
        expected: u32,
    },
    /// A token offset is past the end or inside a UTF-8 sequence.
    Boundary {
        index: usize,
        role: &'static str,
        offset: u32,
    },
    /// A token's offsets are out of order, or a `missing` token has width.
    Malformed { index: usize, role: &'static str },
    /// The tokens end before the source does.
    Short { end: u32, len: u32 },
}

impl fmt::Display for TileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gap {
                index,
                role,
                start,
                expected,
            } => write!(
                f,
                "token {index} (`{role}`) starts at {start}, expected {expected}"
            ),
            Self::Boundary {
                index,
                role,
                offset,
            } => write!(
                f,
                "token {index} (`{role}`) offset {offset} is not a character boundary of the block"
            ),
            Self::Malformed { index, role } => write!(
                f,
                "token {index} (`{role}`) is malformed: offsets out of order or a missing token with width"
            ),
            Self::Short { end, len } => {
                write!(f, "the tokens end at {end} but the block is {len} bytes")
            }
        }
    }
}

pub(super) fn walk_children(
    children: &[PageNode],
    visit: &mut dyn FnMut(&'static str, &PageToken),
) {
    for node in children {
        match node {
            PageNode::Element(element) => {
                visit("lt-name", &element.lt_name);
                for attr in &element.attrs {
                    visit("name", &attr.name);
                    if let Some(eq) = &attr.eq {
                        visit("eq", eq);
                    }
                    if let Some(value) = &attr.value {
                        if let Some(open_quote) = &value.open_quote {
                            visit("open-quote", open_quote);
                        }
                        visit("value", &value.content);
                        if let Some(close_quote) = &value.close_quote {
                            visit("close-quote", close_quote);
                        }
                    }
                }
                if let Some(slash) = &element.slash {
                    visit("slash", slash);
                }
                visit("gt", &element.gt);
                walk_children(&element.children, visit);
                if let PageClose::Tag { lt_slash_name, gt } = &element.close {
                    visit("lt-slash-name", lt_slash_name);
                    visit("gt", gt);
                }
            }
            PageNode::Interpolation(node) => {
                visit("open", &node.open);
                visit("content", &node.content);
                visit("close", &node.close);
            }
            PageNode::Text(token) => visit("text", token),
            PageNode::Comment(token) => visit("comment", token),
            PageNode::Cdata(token) => visit("cdata", token),
            PageNode::ProcessingInstruction(token) => visit("pi", token),
            PageNode::Unexpected(token) => visit("unexpected", token),
        }
    }
}

impl SurfacePage {
    /// Check that the tokens tile `source` exactly, in render order.
    ///
    /// # Errors
    ///
    /// The first token that leaves a gap, overlaps, or splits a character,
    /// or [`TileError::Short`] when bytes are left over.
    pub fn check_tiles(&self, source: &str) -> Result<(), TileError> {
        let mut expected = 0u32;
        let mut index = 0usize;
        let mut first = Ok(());
        self.for_each_token(&mut |role, token| {
            if first.is_ok() {
                first = check_token(source, index, role, token, expected);
                expected = token.end;
            }
            index += 1;
        });
        first?;
        let len = source.len() as u32;
        if expected != len {
            return Err(TileError::Short { end: expected, len });
        }
        Ok(())
    }
}

fn check_token(
    source: &str,
    index: usize,
    role: &'static str,
    token: &PageToken,
    expected: u32,
) -> Result<(), TileError> {
    if token.start != expected {
        return Err(TileError::Gap {
            index,
            role,
            start: token.start,
            expected,
        });
    }
    let ordered = token.start <= token.text && token.text <= token.end;
    if !ordered || (token.missing && token.text != token.end) {
        return Err(TileError::Malformed { index, role });
    }
    for offset in [token.text, token.end] {
        if !source.is_char_boundary(offset as usize) {
            return Err(TileError::Boundary {
                index,
                role,
                offset,
            });
        }
    }
    Ok(())
}
