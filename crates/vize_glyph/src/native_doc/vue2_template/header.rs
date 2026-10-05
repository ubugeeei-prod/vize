//! Conservative plain original Vue 2 attributes in the same layout visit.

use vize_l0::{Allocator, SourceBlock, Vec};
use vize_l1::{Attribute, Element};

use super::{Cursor, Doc, NativeVue2SfcRefusal, TemplateRefusal, span, verbatim};
use crate::native_doc::Line;

pub(super) fn open<'a>(
    element: &Element<'a>,
    block: SourceBlock<'a>,
    parts: &mut Vec<'a, Doc<'a>>,
    cursor: &mut Cursor<'a>,
    allocator: &'a Allocator,
    depth: usize,
) -> Result<(), NativeVue2SfcRefusal> {
    if element.open.is_verbatim()
        || !["div", "span", "p", "section", "a", "br", "input"]
            .iter()
            .any(|name| element.tag().eq_ignore_ascii_case(name))
    {
        return Err(NativeVue2SfcRefusal::UnsupportedElement {
            span: span(block, element.open.lt_name.text)?,
        });
    }
    cursor.token(&element.open.lt_name)?;
    parts.push(Doc::text(element.open.lt_name.leading));
    let mut opening = Vec::new_in(&allocator);
    opening.push(Doc::text(element.open.lt_name.text));
    let mut attributes = Vec::new_in(&allocator);
    for attribute in &element.open.attrs {
        attributes.push(Doc::line(Line::Space));
        attribute_document(attribute, block, &mut attributes, cursor, allocator)?;
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

fn attribute_document<'a>(
    attribute: &Attribute<'a>,
    block: SourceBlock<'a>,
    parts: &mut Vec<'a, Doc<'a>>,
    cursor: &mut Cursor<'a>,
    allocator: &'a Allocator,
) -> Result<(), NativeVue2SfcRefusal> {
    cursor.trivia(&attribute.name)?;
    let name = attribute.name.text;
    if name.starts_with("v-")
        || name.starts_with([':', '@', '#', '.'])
        || [
            "slot",
            "slot-scope",
            "scope",
            "is",
            "inline-template",
            "key",
            "ref",
            "ref-in-for",
        ]
        .iter()
        .any(|reserved| name.eq_ignore_ascii_case(reserved))
    {
        return Err(NativeVue2SfcRefusal::UnsupportedAttribute {
            span: span(block, name)?,
        });
    }
    let mut value = Vec::new_in(&allocator);
    value.push(Doc::text(name));
    if let Some(eq) = &attribute.eq {
        cursor.trivia(eq)?;
        value.push(Doc::text(eq.text));
    }
    if let Some(original) = &attribute.value {
        if attribute.eq.is_none() {
            return Err(TemplateRefusal::Recovered {
                offset: cursor.offset,
            }
            .into());
        }
        if let Some(open) = &original.open_quote {
            cursor.trivia(open)?;
            value.push(Doc::text(open.text));
            cursor.token(&original.content)?;
            verbatim(&mut value, &original.content);
        } else {
            cursor.trivia(&original.content)?;
            value.push(Doc::text(original.content.text));
        }
        if let Some(close) = &original.close_quote {
            cursor.token(close)?;
            verbatim(&mut value, close);
        }
    }
    parts.push(Doc::concat(value));
    Ok(())
}
