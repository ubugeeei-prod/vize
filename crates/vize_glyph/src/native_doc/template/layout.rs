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
    cursor.token(&element.open.lt_name)?;
    parts.push(Doc::text(element.open.lt_name.leading));
    let mut opening = Vec::new_in(&allocator);
    opening.push(Doc::text(element.open.lt_name.text));
    let mut attributes = Vec::new_in(&allocator);
    for attribute in &element.open.attrs {
        attributes.push(Doc::line(Line::Space));
        attribute_document(attribute, &mut attributes, cursor, allocator, policy)?;
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
            policy.value(head.is_some(), &value.content, offset)?;
            verbatim(&mut value_parts, &value.content);
        } else {
            let offset = cursor.offset;
            cursor.trivia(&value.content)?;
            policy.value(head.is_some(), &value.content, offset)?;
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
