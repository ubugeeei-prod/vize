//! Original ordinary static attributes must exhaust before an Element body.

use crate::lang::js::file::native::NativeTemplateIssueKind as Kind;
use crate::op::Attribute;
use vize_l0::{SourceBlock, Span, Vec};
use vize_l1::{
    AttrValue,
    dialect::vue3::VueDirectives,
    markup::{NativeAttribute, NativeElement, directive::DirectiveSyntax},
};

pub(super) fn construct<'a>(
    original: &NativeElement<'_, 'a>,
) -> Result<Vec<'a, Attribute<'a>>, Kind> {
    let mut attributes = Vec::new_in(&original.component().allocator());
    // The original iterator is private to this fused helper. Only its actual
    // normal end can return a header to the existing Element factory.
    for attribute in original.attributes() {
        if !core::ptr::eq(attribute.component(), original.component())
            || !core::ptr::eq(attribute.element(), original.surface())
            || attribute.ordinal() != attributes.len()
            || !original
                .surface()
                .open
                .attrs
                .get(attribute.ordinal())
                .is_some_and(|surface| core::ptr::eq(surface, attribute.surface()))
        {
            return Err(Kind::InvalidEvent);
        }
        let attribute = ordinary(attribute)?;
        if attributes
            .iter()
            .any(|previous: &Attribute<'_>| previous.name == attribute.name)
        {
            return Err(Kind::UnsupportedChild);
        }
        attributes.push(attribute);
    }
    Ok(attributes)
}

fn ordinary<'a>(original: NativeAttribute<'_, 'a>) -> Result<Attribute<'a>, Kind> {
    let surface = original.surface();
    let block = original.component().block();
    let name = surface.name.text;
    let name_span = block.span_of(name).ok_or(Kind::InvalidEvent)?;
    if surface.name.is_missing()
        || name.is_empty()
        || matches!(name, "class" | "style" | "key" | "ref" | "is")
        || !matches!(VueDirectives.decompose(name, name_span.start), Ok(None))
    {
        return Err(Kind::UnsupportedChild);
    }
    let (value, end) = match (&surface.eq, &surface.value) {
        (None, None) => (None, name_span.end),
        (Some(eq), Some(value)) if !eq.is_missing() && eq.text == "=" => {
            let eq_span = block.span_of(eq.text).ok_or(Kind::InvalidEvent)?;
            if eq_span.start < name_span.end {
                return Err(Kind::InvalidEvent);
            }
            let end = value_end(value, block, eq_span.end)?;
            (Some(value.content.text), end)
        }
        _ => return Err(Kind::UnsupportedChild),
    };
    Ok(Attribute {
        name,
        value,
        span: Span::new(name_span.start, end),
    })
}

fn value_end(value: &AttrValue<'_>, block: SourceBlock<'_>, after: u32) -> Result<u32, Kind> {
    if value.content.is_missing() || value.content.text.contains('&') {
        return Err(Kind::UnsupportedChild);
    }
    let content = block
        .span_of(value.content.text)
        .ok_or(Kind::InvalidEvent)?;
    match (&value.open_quote, &value.close_quote) {
        (None, None) if content.start >= after => Ok(content.end),
        (Some(open), Some(close))
            if !open.is_missing()
                && !close.is_missing()
                && matches!(open.text, "\"" | "'")
                && open.text == close.text =>
        {
            let open = block.span_of(open.text).ok_or(Kind::InvalidEvent)?;
            let close = block.span_of(close.text).ok_or(Kind::InvalidEvent)?;
            if open.start < after || content.start < open.end || close.start < content.end {
                return Err(Kind::InvalidEvent);
            }
            Ok(close.end)
        }
        _ => Err(Kind::UnsupportedChild),
    }
}
