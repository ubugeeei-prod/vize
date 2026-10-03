//! Original-header `a11y/iframe-has-title`, with native L1 entity decoding.

use vize_l0::diag::{Advisory, Diagnostic, DiagnosticPart, MessageLookup, PartKind, Stage};
use vize_l1::markup::{
    NativeElement,
    entity::{EntityContext, decode_one},
};

use super::{IFRAME_HAS_TITLE_RULE, NativeLintFinding, NativeLintRefusal, header};

pub(super) fn check(
    element: &NativeElement<'_, '_>,
    messages: &impl MessageLookup,
) -> Result<Option<NativeLintFinding>, NativeLintRefusal> {
    let range = header::opening_range(element)?;
    let mut has_title = false;
    for original in element.attributes() {
        has_title |= match header::binding(element, &original)? {
            header::Binding::Static {
                name: "title",
                value,
            } => value.map(has_non_whitespace).transpose()?.unwrap_or(false),
            header::Binding::Bind { name: "title" } => true,
            _ => false,
        };
    }
    if element.surface().tag() != "iframe" || has_title {
        return Ok(None);
    }
    let diagnostic = Diagnostic::new(
        Advisory::Warning,
        Stage::Surface,
        range,
        messages.lookup("a11y/iframe-has-title.message").as_ref(),
    )
    .with_part(DiagnosticPart::new(
        PartKind::Help,
        range,
        messages.lookup("a11y/iframe-has-title.help").as_ref(),
    ));
    Ok(Some(NativeLintFinding {
        rule_name: IFRAME_HAS_TITLE_RULE,
        diagnostic,
    }))
}

// Inspect the decoded scalar stream without allocating or parsing expressions.
// Unknown references remain literal, as in the original attribute grammar.
fn has_non_whitespace(mut value: &str) -> Result<bool, NativeLintRefusal> {
    while let Some(ch) = value.chars().next() {
        if ch == '&'
            && let Some((decoded, consumed)) =
                decode_one(value.as_bytes(), EntityContext::Attribute)
        {
            let mut non_whitespace = false;
            decoded.for_each(|ch| non_whitespace |= !ch.is_whitespace());
            if non_whitespace {
                return Ok(true);
            }
            value = value
                .get(consumed..)
                .ok_or(NativeLintRefusal::SourceMismatch)?;
        } else {
            if !ch.is_whitespace() {
                return Ok(true);
            }
            value = value
                .get(ch.len_utf8()..)
                .ok_or(NativeLintRefusal::SourceMismatch)?;
        }
    }
    Ok(false)
}
