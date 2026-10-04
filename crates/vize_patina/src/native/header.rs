//! Checked original element ranges and header binding names, shared by syntax rules.

use vize_l0::Span;
use vize_l1::{
    ElementClose,
    markup::{NativeAttribute, NativeElement},
};

use super::{NativeLintRefusal, attribute};

mod checked;
use checked::binding_with_modifiers;

pub(super) fn opening_range(element: &NativeElement<'_, '_>) -> Result<Span, NativeLintRefusal> {
    let block = element.component().block();
    let surface = element.surface();
    attribute::token(block, &surface.open.lt_name)?;
    attribute::token(block, &surface.open.gt)?;
    if let Some(slash) = &surface.open.slash {
        attribute::token(block, slash)?;
    }
    let start = attribute::span(block, surface.open.lt_name.text)?.start;
    let opening = Span::new(start, attribute::span(block, surface.open.gt.text)?.end);
    let end = match &surface.close {
        ElementClose::Present(close) => {
            attribute::token(block, &close.lt_slash_name)?;
            attribute::token(block, &close.gt)?;
            attribute::span(block, close.gt.text)?.end
        }
        ElementClose::NotExpected => opening.end,
        ElementClose::Missing | ElementClose::Implicit => return Err(NativeLintRefusal::Hole),
    };
    let authored = Span::new(start, end);
    if !block.contains_block_span(authored) {
        return Err(NativeLintRefusal::SourceMismatch);
    }
    Ok(opening)
}

pub(super) enum Binding<'a> {
    Static {
        name: &'a str,
        value: Option<&'a str>,
        range: Span,
    },
    Bind {
        name: &'a str,
        argument_range: Span,
        range: Span,
    },
    Other {
        full_directive_name: Option<&'a str>,
        range: Span,
    },
}

pub(super) fn binding<'a>(
    element: &NativeElement<'_, 'a>,
    original: &NativeAttribute<'_, 'a>,
) -> Result<Binding<'a>, NativeLintRefusal> {
    binding_with_modifiers::<false>(element, original)
}

/// New full-result consumers refuse parser errors from empty modifier segments.
/// Prior syntax rule admission keeps its original const-false behavior.
pub(super) fn strict_binding<'a>(
    element: &NativeElement<'_, 'a>,
    original: &NativeAttribute<'_, 'a>,
) -> Result<Binding<'a>, NativeLintRefusal> {
    binding_with_modifiers::<true>(element, original)
}
