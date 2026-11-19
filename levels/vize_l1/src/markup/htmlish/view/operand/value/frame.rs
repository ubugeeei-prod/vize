use super::{AttributeOperandError, AttributeView, Origin};
use crate::embed::SourceError;
use vize_l0::Span;

pub(super) struct Frame {
    pub(super) value: Span,
    pub(super) equals: Span,
    pub(super) quotes: Option<(Span, Span)>,
}
impl Frame {
    pub(super) fn original(
        origin: &Origin<'_>,
        attribute: &AttributeView<'_, '_>,
    ) -> Result<Self, AttributeOperandError> {
        let surface = attribute.surface();
        let equals = surface
            .eq
            .as_ref()
            .ok_or(AttributeOperandError::IncompleteValue)?;
        let value = surface
            .value
            .as_ref()
            .ok_or(AttributeOperandError::IncompleteValue)?;
        let span = |text| {
            origin
                .block
                .span_of(text)
                .ok_or(AttributeOperandError::Source(
                    SourceError::InvalidAuthoredSpan,
                ))
        };
        let equals_span = span(equals.text)?;
        if equals.text != "=" || equals_span.start < origin.name_span.end {
            return Err(AttributeOperandError::IncompleteValue);
        }
        let (full_value, quotes) = match (&value.open_quote, &value.close_quote) {
            (None, None) if origin.value_span.start >= equals_span.end => (origin.value_span, None),
            (Some(open), Some(close))
                if matches!(open.text, "\"" | "'") && open.text == close.text =>
            {
                let open = span(open.text)?;
                let close = span(close.text)?;
                if open.start < equals_span.end
                    || origin.value_span.start != open.end
                    || origin.value_span.end != close.start
                {
                    return Err(AttributeOperandError::IncompleteValue);
                }
                (Span::new(open.start, close.end), Some((open, close)))
            }
            _ => return Err(AttributeOperandError::IncompleteValue),
        };
        Ok(Self {
            value: full_value,
            equals: equals_span,
            quotes,
        })
    }
}
