//! The supported Vue 3 surface-to-document path. No product parser is called.

use vize_l0::{Allocator, Vec};
use vize_l1::dialect::vue3::{VueDirectives, surface::ComponentParse};
use vize_l1::markup::DirectiveSyntax;
use vize_l1::{Attribute, Element, ElementClose, SurfaceChild, Token};

use super::{Doc, Line};

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

/// Lay out plain opening-tag attributes from a retained native Vue 3 parse.
///
/// The caller parses once with L1's native `parse_component` entry point.
/// Content whitespace, comments, entities, values, quotes and attribute order
/// stay verbatim. This API refuses recovered/repaired trees, directives and
/// interpolation rather than using a legacy parser or an incomplete embed.
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

struct Cursor<'a> {
    source: &'a str,
    offset: usize,
}

impl Cursor<'_> {
    fn token(&mut self, token: &Token<'_>) -> Result<(), TemplateRefusal> {
        if token.is_missing() {
            return Err(TemplateRefusal::Recovered {
                offset: self.offset,
            });
        }
        for piece in [token.leading, token.text] {
            let end =
                self.offset
                    .checked_add(piece.len())
                    .ok_or(TemplateRefusal::SourceMismatch {
                        offset: self.offset,
                    })?;
            let expected =
                self.source
                    .get(self.offset..end)
                    .ok_or(TemplateRefusal::SourceMismatch {
                        offset: self.offset,
                    })?;
            if !piece.is_empty() && !core::ptr::eq(expected.as_ptr(), piece.as_ptr()) {
                return Err(TemplateRefusal::SourceMismatch {
                    offset: self.offset,
                });
            }
            self.offset = end;
        }
        Ok(())
    }

    fn trivia(&mut self, token: &Token<'_>) -> Result<(), TemplateRefusal> {
        if !token.leading.bytes().all(|byte| byte.is_ascii_whitespace()) {
            return Err(TemplateRefusal::Recovered {
                offset: self.offset,
            });
        }
        self.token(token)
    }
}

fn verbatim<'a>(parts: &mut Vec<'a, Doc<'a>>, token: &Token<'a>) {
    parts.push(Doc::text(token.leading));
    parts.push(Doc::text(token.text));
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
    cursor.token(&element.open.lt_name)?;
    parts.push(Doc::text(element.open.lt_name.leading));
    let mut opening = Vec::new_in(&allocator);
    opening.push(Doc::text(element.open.lt_name.text));
    let mut attributes = Vec::new_in(&allocator);
    for attribute in &element.open.attrs {
        attributes.push(Doc::line(Line::Space));
        attribute_document(attribute, &mut attributes, cursor, allocator)?;
    }
    if !attributes.is_empty() {
        opening.push(Doc::concat(attributes).indent(depth + 1, allocator));
    }
    let line = if element.open.slash.is_some() {
        Line::Space
    } else {
        Line::Empty
    };
    if !element.open.attrs.is_empty() || element.open.slash.is_some() {
        opening.push(Doc::line(line).indent(depth, allocator));
    }
    if let Some(slash) = &element.open.slash {
        cursor.trivia(slash)?;
        opening.push(Doc::text(slash.text));
    }
    cursor.trivia(&element.open.gt)?;
    opening.push(Doc::text(element.open.gt.text));
    parts.push(Doc::concat(opening).group(allocator));
    children(&element.children, parts, cursor, allocator, depth + 1)?;
    match &element.close {
        ElementClose::Present(close) => {
            cursor.token(&close.lt_slash_name)?;
            verbatim(parts, &close.lt_slash_name);
            cursor.token(&close.gt)?;
            verbatim(parts, &close.gt);
        }
        ElementClose::NotExpected => {}
        ElementClose::Missing | ElementClose::Implicit => {
            return Err(TemplateRefusal::Recovered {
                offset: cursor.offset,
            });
        }
    }
    Ok(())
}

fn attribute_document<'a>(
    attribute: &Attribute<'a>,
    parts: &mut Vec<'a, Doc<'a>>,
    cursor: &mut Cursor<'a>,
    allocator: &'a Allocator,
) -> Result<(), TemplateRefusal> {
    let start = cursor.offset.saturating_add(attribute.name.leading.len());
    let offset =
        u32::try_from(start).map_err(|_| TemplateRefusal::SourceMismatch { offset: start })?;
    match VueDirectives.decompose(attribute.name.text, offset) {
        Ok(None) => {}
        Ok(Some(_)) | Err(_) => {
            return Err(TemplateRefusal::Unsupported {
                offset: start,
                syntax: UnsupportedSyntax::Directive,
            });
        }
    }
    cursor.trivia(&attribute.name)?;
    let mut value_parts = Vec::new_in(&allocator);
    value_parts.push(Doc::text(attribute.name.text));
    if let Some(eq) = &attribute.eq {
        cursor.trivia(eq)?;
        value_parts.push(Doc::text(eq.text));
    }
    if let Some(value) = &attribute.value {
        if let Some(open) = &value.open_quote {
            cursor.trivia(open)?;
            value_parts.push(Doc::text(open.text));
            cursor.token(&value.content)?;
            verbatim(&mut value_parts, &value.content);
        } else {
            cursor.trivia(&value.content)?;
            value_parts.push(Doc::text(value.content.text));
        }
        if let Some(close) = &value.close_quote {
            cursor.token(close)?;
            verbatim(&mut value_parts, close);
        }
    }
    parts.push(Doc::concat(value_parts));
    Ok(())
}
