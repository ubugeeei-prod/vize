use super::super::attribute;
use super::{NativeChildFactError, NativeHeaderFacts, NativeInterpolationFact, NativeLintRefusal};
use vize_l0::Span;
use vize_l1::{ElementClose, SurfaceChild, markup::NativeChild};

pub(super) fn observe(
    header: &NativeHeaderFacts<'_, '_>,
    child: &NativeChild<'_, '_>,
    header_key: u32,
) -> Result<Option<(u32, NativeInterpolationFact)>, NativeChildFactError> {
    let interpolation = match child.surface() {
        SurfaceChild::Interpolation(interpolation) => interpolation,
        SurfaceChild::Unexpected(_) => return Err(NativeChildFactError::UnexpectedChild),
        _ => return Ok(None),
    };
    if interpolation.is_raw_html() {
        return Err(NativeChildFactError::RawInterpolation);
    }
    let block = child.component().block();
    for token in [
        &interpolation.open,
        &interpolation.content,
        &interpolation.close,
    ] {
        attribute::token(block, token)?;
    }
    if interpolation.open.text != "{{" || interpolation.close.text != "}}" {
        return Err(NativeLintRefusal::SourceMismatch.into());
    }
    let open = attribute::span(block, interpolation.open.text)?;
    // Validate even zero-width content against the original source pointer.
    let content = attribute::span(block, interpolation.content.text)?;
    let close = attribute::span(block, interpolation.close.text)?;
    let body_start = attribute::span(block, header.original().open.gt.text)?.end;
    let body_end = match &header.original().close {
        ElementClose::Present(close) => attribute::span(block, close.lt_slash_name.text)?.start,
        _ => return Err(NativeLintRefusal::Hole.into()),
    };
    if open.end != content.start
        || content.end != close.start
        || open.start < body_start
        || close.end > body_end
    {
        return Err(NativeLintRefusal::SourceMismatch.into());
    }
    let ordinal = u32::try_from(child.ordinal()).map_err(|_| NativeLintRefusal::SourceMismatch)?;
    Ok(Some((
        ordinal,
        NativeInterpolationFact {
            header: header_key,
            span: Span::new(open.start, close.end),
            content,
        },
    )))
}
