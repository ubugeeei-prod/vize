//! The supported Vue 3 surface-to-document path. No product parser is called.

use vize_l0::{Allocator, Vec};
use vize_l1::dialect::vue3::surface::ComponentParse;
use vize_l1::{Element, SurfaceChild};

use super::{Doc, Line};

#[path = "template/cursor.rs"]
mod cursor;
#[path = "template/layout.rs"]
mod layout;
pub(super) use cursor::{Cursor, verbatim};
pub(super) use layout::{close_element, open_element};

/// Work still requiring typed dialect/embed formatting providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsupportedSyntax {
    Directive,
    Interpolation,
    NestingLimit,
}

/// Refusal never consumes or replaces the caller's retained parse observations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateRefusal {
    Recovered {
        offset: usize,
    },
    Unsupported {
        offset: usize,
        syntax: UnsupportedSyntax,
    },
    SourceMismatch {
        offset: usize,
    },
}

/// The document borrows the original parse and source. Its construction does
/// not constitute a complete native product/dialect acceptance receipt.
#[derive(Debug)]
pub struct TemplateDocument<'p, 'a> {
    original: &'p ComponentParse<'a>,
    document: Doc<'a>,
}

impl<'p, 'a> TemplateDocument<'p, 'a> {
    pub fn original(&self) -> &'p ComponentParse<'a> {
        self.original
    }

    pub fn document(&self) -> &Doc<'a> {
        &self.document
    }
}

/// Lay out plain and typed directive attributes from a retained native Vue 3 parse.
///
/// The caller parses once with L1's native `parse_component` entry point.
/// Content whitespace, comments, entities, values, quotes and attribute order
/// stay verbatim. Full `v-name:arg` and shorthand `:arg`, `.arg`, `@arg` and `#arg`
/// heads use the shared typed Vue syntax without classifying directive semantics.
/// Dynamic arguments retain their complete brackets and original expression bytes.
/// Full heads also admit complete names without an argument, preserving modifier runs.
/// This API refuses recovered/repaired trees, incomplete heads, missing shorthand arguments and
/// interpolation; it never calls a legacy parser or parses values.
/// Token source custody is checked in the same traversal that constructs Doc.
pub fn template_document<'p, 'a>(
    original: &'p ComponentParse<'a>,
    allocator: &'a Allocator,
) -> Result<TemplateDocument<'p, 'a>, TemplateRefusal> {
    if let Some(error) = original.errors.first() {
        return Err(TemplateRefusal::Recovered {
            offset: error.offset as usize,
        });
    }
    if original.authored.is_some() || !original.unsupported.is_empty() {
        return Err(TemplateRefusal::Recovered { offset: 0 });
    }
    let mut cursor = Cursor {
        source: original.tree.source,
        offset: 0,
    };
    let mut parts = Vec::new_in(&allocator);
    children(
        &original.tree.children,
        &mut parts,
        &mut cursor,
        allocator,
        0,
    )?;
    if cursor.offset != cursor.source.len() {
        return Err(TemplateRefusal::SourceMismatch {
            offset: cursor.offset,
        });
    }
    Ok(TemplateDocument {
        original,
        document: Doc::concat(parts),
    })
}

fn children<'a>(
    nodes: &[SurfaceChild<'a>],
    parts: &mut Vec<'a, Doc<'a>>,
    cursor: &mut Cursor<'a>,
    allocator: &'a Allocator,
    depth: usize,
) -> Result<(), TemplateRefusal> {
    if depth > 128 {
        return Err(TemplateRefusal::Unsupported {
            offset: cursor.offset,
            syntax: UnsupportedSyntax::NestingLimit,
        });
    }
    for node in nodes {
        match node {
            SurfaceChild::Element(element) => {
                element_document(element, parts, cursor, allocator, depth)?
            }
            SurfaceChild::Interpolation(_) => {
                return Err(TemplateRefusal::Unsupported {
                    offset: cursor.offset,
                    syntax: UnsupportedSyntax::Interpolation,
                });
            }
            SurfaceChild::Unexpected(_) => {
                return Err(TemplateRefusal::Recovered {
                    offset: cursor.offset,
                });
            }
            SurfaceChild::Text(token)
            | SurfaceChild::Comment(token)
            | SurfaceChild::Cdata(token)
            | SurfaceChild::ProcessingInstruction(token) => {
                cursor.token(token)?;
                verbatim(parts, token);
            }
        }
    }
    Ok(())
}

fn element_document<'a>(
    element: &Element<'a>,
    parts: &mut Vec<'a, Doc<'a>>,
    cursor: &mut Cursor<'a>,
    allocator: &'a Allocator,
    depth: usize,
) -> Result<(), TemplateRefusal> {
    open_element(element, parts, cursor, allocator, depth)?;
    children(&element.children, parts, cursor, allocator, depth + 1)?;
    close_element(element, parts, cursor)
}
