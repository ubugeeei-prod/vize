//! Original ordinary static attributes must exhaust before an Element body.

use super::super::{NativeVisibility, handler::PreparedHandler};
use crate::artifact::RegionBuilder;
use crate::file::region::FileRegion;
use crate::lang::js::file::native::NativeTemplateIssueKind as Kind;
use crate::op::Attribute;
use core::ops::DerefMut;
use vize_l0::{SourceBlock, Span, Vec};
use vize_l1::{
    AttrValue,
    dialect::vue3::VueDirectives,
    markup::{
        NativeAttribute, NativeElement, NativeTemplateComponent,
        directive::{DirectivePrefix, DirectiveSyntax},
    },
};

pub(super) struct Header<'a> {
    pub(super) attributes: Vec<'a, Attribute<'a>>,
    pub(super) handlers: alloc::vec::Vec<PreparedHandler<'a>>,
}

pub(super) fn construct<'a: 'b, 'b, R>(
    original: &NativeElement<'_, 'a>,
    selected: &NativeTemplateComponent<'a>,
    region: &mut FileRegion<'_, 'b, 'a, R, NativeVisibility>,
) -> Result<Header<'a>, Kind>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
{
    let mut attributes = Vec::new_in(&original.component().allocator());
    let mut handlers: alloc::vec::Vec<PreparedHandler<'a>> = alloc::vec::Vec::new();
    // One actual full header iterator. Only its normal end permits the Element
    // factory and original callback; pending owners remain parked on refusal.
    for (ordinal, attribute) in original.attributes().enumerate() {
        if !core::ptr::eq(selected.component(), original.component())
            || !core::ptr::eq(attribute.component(), original.component())
            || !core::ptr::eq(attribute.element(), original.surface())
            || attribute.ordinal() != ordinal
            || !original
                .surface()
                .open
                .attrs
                .get(ordinal)
                .is_some_and(|surface| core::ptr::eq(surface, attribute.surface()))
        {
            return Err(Kind::InvalidEvent);
        }
        let block = original.component().block();
        let name_span = block
            .span_of(attribute.surface().name.text)
            .ok_or(Kind::InvalidEvent)?;
        match VueDirectives.decompose(attribute.surface().name.text, name_span.start) {
            Ok(None) => {
                let attribute = ordinary(attribute)?;
                if attributes
                    .iter()
                    .any(|previous: &Attribute<'_>| previous.name == attribute.name)
                {
                    return Err(Kind::UnsupportedChild);
                }
                attributes.push(attribute);
            }
            Ok(Some(directive))
                if directive.prefix == DirectivePrefix::On
                    || (directive.prefix == DirectivePrefix::Full
                        && directive.name.slice(block.root_source()) == "on") =>
            {
                let handler = region.prepare_handler(selected, attribute)?;
                if handlers
                    .iter()
                    .any(|previous| previous.name == handler.name)
                {
                    return Err(Kind::UnsupportedChild);
                }
                handlers.push(handler);
            }
            Ok(Some(_)) | Err(_) => return Err(Kind::UnsupportedChild),
        }
    }
    Ok(Header {
        attributes,
        handlers,
    })
}

fn ordinary<'a>(original: NativeAttribute<'_, 'a>) -> Result<Attribute<'a>, Kind> {
    let surface = original.surface();
    let block = original.component().block();
    let name = surface.name.text;
    let name_span = block.span_of(name).ok_or(Kind::InvalidEvent)?;
    if surface.name.is_missing()
        || name.is_empty()
        || matches!(name, "class" | "style" | "key" | "ref" | "is")
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
