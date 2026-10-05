//! Shared opening/closing layout, retaining original attribute and token bytes.

use vize_l0::{Allocator, SourceRoot, Vec};
use vize_l1::dialect::vue3::VueDirectives;
use vize_l1::markup::DirectiveSyntax;
use vize_l1::{Attribute, Element, ElementClose};

use super::{
    AttributeValuePolicy, Cursor, Doc, Line, TemplateRefusal, UnsupportedSyntax, verbatim,
};

pub(in crate::native_doc) fn open_element<'a, P: AttributeValuePolicy<'a>>(
    element: &Element<'a>,
    parts: &mut Vec<'a, Doc<'a>>,
    cursor: &mut Cursor<'a>,
    allocator: &'a Allocator,
    depth: usize,
    policy: &P,
) -> Result<(), P::Refusal> {
    open_element_with(
        element,
        element.open.attrs.iter(),
        parts,
        cursor,
        allocator,
        depth,
        |attribute, parts, cursor| attribute_document(attribute, parts, cursor, allocator, policy),
    )
}

/// Share layout while keeping the receiver's actual original attribute iterator.
pub(in crate::native_doc) fn open_element_with<'a, T, R: From<TemplateRefusal>>(
    element: &Element<'a>,
    attributes: impl IntoIterator<Item = T>,
    parts: &mut Vec<'a, Doc<'a>>,
    cursor: &mut Cursor<'a>,
    allocator: &'a Allocator,
    depth: usize,
    mut attribute_document: impl FnMut(T, &mut Vec<'a, Doc<'a>>, &mut Cursor<'a>) -> Result<(), R>,
) -> Result<(), R> {
    cursor.token(&element.open.lt_name)?;
    parts.push(Doc::text(element.open.lt_name.leading));
    let mut opening = Vec::new_in(&allocator);
    opening.push(Doc::text(element.open.lt_name.text));
    let mut attribute_docs = Vec::new_in(&allocator);
    for attribute in attributes {
        attribute_docs.push(Doc::line(Line::Space));
        attribute_document(attribute, &mut attribute_docs, cursor)?;
    }
    if !attribute_docs.is_empty() {
        opening.push(Doc::concat(attribute_docs).indent(depth + 1, allocator));
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
    Ok(())
}

pub(in crate::native_doc) fn close_element<'a>(
    element: &Element<'a>,
    parts: &mut Vec<'a, Doc<'a>>,
    cursor: &mut Cursor<'a>,
) -> Result<(), TemplateRefusal> {
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

fn attribute_document<'a, P: AttributeValuePolicy<'a>>(
    attribute: &Attribute<'a>,
    parts: &mut Vec<'a, Doc<'a>>,
    cursor: &mut Cursor<'a>,
    allocator: &'a Allocator,
    policy: &P,
) -> Result<(), P::Refusal> {
    let start = cursor.offset.saturating_add(attribute.name.leading.len());
    let offset =
        u32::try_from(start).map_err(|_| TemplateRefusal::SourceMismatch { offset: start })?;
    let block = SourceRoot::new(cursor.source)
        .and_then(|root| root.block(attribute.name.text, offset))
        .map_err(|_| TemplateRefusal::SourceMismatch { offset: start })?;
    let head = VueDirectives
        .decompose(block.source(), block.start())
        .map_err(|_| TemplateRefusal::Unsupported {
            offset: start,
            syntax: UnsupportedSyntax::Directive,
        })?;
    let name = super::super::directive::name_document(block, head, allocator)?;
    attribute_with_name(
        attribute,
        name,
        parts,
        cursor,
        allocator,
        |content, offset| {
            policy.value(head.is_some(), content, offset)?;
            Ok(None)
        },
    )
}

/// The supplied name Doc has already been checked against this original token.
/// The value callback runs at the same checked content visit before projection.
pub(in crate::native_doc) fn attribute_with_name<'a, R: From<TemplateRefusal>>(
    attribute: &Attribute<'a>,
    name: Doc<'a>,
    parts: &mut Vec<'a, Doc<'a>>,
    cursor: &mut Cursor<'a>,
    allocator: &'a Allocator,
    mut value_document: impl FnMut(&vize_l1::Token<'a>, usize) -> Result<Option<Doc<'a>>, R>,
) -> Result<(), R> {
    cursor.trivia(&attribute.name)?;
    let mut value_parts = Vec::new_in(&allocator);
    value_parts.push(name);
    if let Some(eq) = &attribute.eq {
        cursor.trivia(eq)?;
        value_parts.push(Doc::text(eq.text));
    }
    if let Some(value) = &attribute.value {
        if let Some(open) = &value.open_quote {
            cursor.trivia(open)?;
            value_parts.push(Doc::text(open.text));
            let offset = cursor.offset;
            cursor.token(&value.content)?;
            if let Some(document) = value_document(&value.content, offset)? {
                value_parts.push(Doc::text(value.content.leading));
                value_parts.push(document);
            } else {
                verbatim(&mut value_parts, &value.content);
            }
        } else {
            let offset = cursor.offset;
            cursor.trivia(&value.content)?;
            value_parts.push(
                value_document(&value.content, offset)?
                    .unwrap_or_else(|| Doc::text(value.content.text)),
            );
        }
        if let Some(close) = &value.close_quote {
            cursor.token(close)?;
            verbatim(&mut value_parts, close);
        }
    }
    parts.push(Doc::concat(value_parts));
    Ok(())
}
