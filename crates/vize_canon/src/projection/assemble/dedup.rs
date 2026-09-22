//! Collapsing duplicate authored diagnostics.

use vize_carton::{FxHashSet, String};

use super::AssembledDiagnostic;

/// Identity of an authored diagnostic within one file: after projection,
/// distinct generated positions collapse onto one authored position (a
/// template binding read by the expression statement and by an undefined-name
/// check, a prop checked by its annotation and by the props literal), and the
/// checker reports the same finding at each (#1389). Severity is part of the
/// key so a genuine error+hint pair on one span survives.
type DiagnosticKey = (usize, Option<u32>, String, Option<u8>);

fn key<P>(diagnostic: &AssembledDiagnostic<P>) -> DiagnosticKey {
    (
        diagnostic.start,
        diagnostic.code,
        diagnostic.message.clone(),
        diagnostic.severity,
    )
}

/// Drop duplicates, keeping first-seen order.
///
/// One binding with a literal value is checked twice — by the per-prop
/// annotation and by the whole-props object literal — and TypeScript renders
/// the same complaint differently at the two sites: the annotation keeps the
/// literal (`Type '123' is not assignable …`) while object-property
/// elaboration widens the fresh literal (`Type 'number' is not assignable …`).
/// `vue-tsc` reports the widened form once, so when both spellings of one
/// complaint land on one span only the widened one survives (#4966).
pub(super) fn dedup<P>(diagnostics: Vec<AssembledDiagnostic<P>>) -> Vec<AssembledDiagnostic<P>> {
    let mut seen = FxHashSet::default();
    let mut deduped = Vec::with_capacity(diagnostics.len());
    for diagnostic in diagnostics {
        if seen.insert(key(&diagnostic)) {
            deduped.push(diagnostic);
        }
    }
    deduped.retain(|diagnostic| {
        widen_leading_literal_type(&diagnostic.message).is_none_or(|widened| {
            let mut key = key(diagnostic);
            key.2 = widened;
            !seen.contains(&key)
        })
    });
    deduped
}

/// The message with its leading `Type '<literal>'` widened to the literal's
/// primitive, or `None` when the message does not start with a literal type.
pub fn widen_leading_literal_type(message: &str) -> Option<String> {
    const PREFIX: &str = "Type '";
    const NEEDLE: &str = "' is not assignable to ";
    let rest = message.strip_prefix(PREFIX)?;
    let end = rest.find(NEEDLE)?;
    let widened = widened_primitive_name(&rest[..end])?;
    let mut normalized = String::from(PREFIX);
    normalized.push_str(widened);
    normalized.push_str(&rest[end..]);
    Some(normalized)
}

fn widened_primitive_name(rendered: &str) -> Option<&'static str> {
    if rendered.len() >= 2 && rendered.starts_with('"') && rendered.ends_with('"') {
        return Some("string");
    }
    match rendered {
        "true" | "false" => Some("boolean"),
        _ if rendered.strip_suffix('n').is_some_and(|digits| {
            !digits.is_empty() && digits.chars().all(|ch| ch.is_ascii_digit() || ch == '-')
        }) =>
        {
            Some("bigint")
        }
        _ if rendered.parse::<f64>().is_ok() => Some("number"),
        _ => None,
    }
}
