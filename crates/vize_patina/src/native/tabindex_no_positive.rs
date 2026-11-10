//! Literal tabindex numeric checks over original same-owner attributes.

use std::borrow::Cow;

use vize_l0::{
    SmallVec, Span, String,
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
    let mut names: SmallVec<[(&str, bool); 8]> = SmallVec::new();
    for original in element.attributes() {
        let (name, value, range, bound) = match header::binding(element, &original)? {
            header::Binding::Static { name, value, range } => (name, value, range, false),
            header::Binding::Bind { name, range, .. } => (
                name,
                original
                    .surface()
                    .value
                    .as_ref()
                    .map(|value| value.content.text),
                range,
                true,
            ),
            header::Binding::Other { .. } => continue,
        };
        let permits_static_bound_pair =
            name.eq_ignore_ascii_case("class") || name.eq_ignore_ascii_case("style");
        if names.iter().any(|(previous, previous_bound)| {
            previous.eq_ignore_ascii_case(name)
                && (!permits_static_bound_pair || *previous_bound == bound)
        }) {
            let span = attribute::span(element.component().block(), original.surface().name.text)?;
            return Err(NativeLintRefusal::DuplicateAttribute { span });
        }
        names.push((name, bound));
        let Some(value) = value else {
            continue;
        };
        let positive = if bound {
            crate::attribute_policy::positive_bound_tabindex_literal(value)
        } else {
            positive(value)?
        };
        if !name.eq_ignore_ascii_case("tabindex") || !positive {
            continue;
        }
        findings.push(finding(range, messages));
    }
    Ok(findings)
}

pub(super) fn finding(range: Span, messages: &impl MessageLookup) -> NativeLintFinding {
    Messages::new(messages).finding(range)
}

/// One query's immutable catalog selection, initialized only for a finding.
pub(super) struct Messages {
    message: Cow<'static, str>,
    help: Cow<'static, str>,
}

impl Messages {
    pub(super) fn new(messages: &impl MessageLookup) -> Self {
        Self {
            message: messages.lookup("a11y/tabindex-no-positive.message"),
            help: messages.lookup("a11y/tabindex-no-positive.help"),
        }
    }

    pub(super) fn finding(&self, range: Span) -> NativeLintFinding {
        let diagnostic = Diagnostic::new(
            Advisory::Warning,
            Stage::Surface,
            range,
            self.message.as_ref(),
        )
        .with_part(DiagnosticPart::new(
            PartKind::Help,
            range,
            self.help.as_ref(),
        ));
        NativeLintFinding {
            rule_name: TABINDEX_NO_POSITIVE_RULE,
            diagnostic,
        }
    }
}

fn positive(value: &str) -> Result<bool, NativeLintRefusal> {
    if !needs_decoding(value.as_bytes()) {
        return Ok(crate::attribute_policy::positive_tabindex(value));
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
    Ok(crate::attribute_policy::positive_tabindex(&decoded))
}
