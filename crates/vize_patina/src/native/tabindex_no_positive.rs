//! Static tabindex integer checks over original same-owner attributes.

use vize_l0::{
    SmallVec, String,
    diag::{Advisory, Diagnostic, DiagnosticPart, MessageLookup, PartKind, Stage},
};
use vize_l1::markup::{
    NativeElement,
    entity::{EntityContext, decode_one, needs_decoding},
};

use super::{NativeLintFinding, NativeLintRefusal, TABINDEX_NO_POSITIVE_RULE, attribute, header};

pub(super) fn check(
    element: &NativeElement<'_, '_>,
    messages: &impl MessageLookup,
) -> Result<Vec<NativeLintFinding>, NativeLintRefusal> {
    header::opening_range(element)?;
    let mut findings = Vec::new();
    let mut names: SmallVec<[&str; 8]> = SmallVec::new();
    for original in element.attributes() {
        let header::Binding::Static { name, value, range } = header::binding(element, &original)?
        else {
            continue;
        };
        if names
            .iter()
            .any(|previous| previous.eq_ignore_ascii_case(name))
        {
            let span = attribute::span(element.component().block(), original.surface().name.text)?;
            return Err(NativeLintRefusal::DuplicateAttribute { span });
        }
        names.push(name);
        let Some(value) = value else {
            continue;
        };
        if !name.eq_ignore_ascii_case("tabindex") || !positive(value)? {
            continue;
        }
        let diagnostic = Diagnostic::new(
            Advisory::Warning,
            Stage::Surface,
            range,
            messages
                .lookup("a11y/tabindex-no-positive.message")
                .as_ref(),
        )
        .with_part(DiagnosticPart::new(
            PartKind::Help,
            range,
            messages.lookup("a11y/tabindex-no-positive.help").as_ref(),
        ));
        findings.push(NativeLintFinding {
            rule_name: TABINDEX_NO_POSITIVE_RULE,
            diagnostic,
        });
    }
    Ok(findings)
}

fn positive(value: &str) -> Result<bool, NativeLintRefusal> {
    if !needs_decoding(value.as_bytes()) {
        return Ok(value.parse::<i32>().is_ok_and(|number| number > 0));
    }
    let mut rest = value;
    let mut decoded = String::new("");
    while let Some(ch) = rest.chars().next() {
        let consumed = if ch == '&'
            && let Some((entity, consumed)) = decode_one(rest.as_bytes(), EntityContext::Attribute)
        {
            entity.for_each(|ch| decoded.push(ch));
            consumed
        } else {
            decoded.push(ch);
            ch.len_utf8()
        };
        rest = rest
            .get(consumed..)
            .ok_or(NativeLintRefusal::SourceMismatch)?;
    }
    Ok(decoded.parse::<i32>().is_ok_and(|number| number > 0))
}
