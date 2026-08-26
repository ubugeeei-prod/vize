//! Collapsing exact-duplicate diagnostics at the collection point, shared by
//! the LSP and CLI paths.

use super::Diagnostic;
use vize_carton::{FxHashSet, String};

/// Identity key for deduplicating diagnostics — (file, line, column, code,
/// message). After source mapping, distinct virtual positions can collapse to
/// the same original position: a template binding (e.g. an undefined name in an
/// interpolation) is referenced more than once in the generated virtual TS —
/// once by the normal template-expression statement and once by the dedicated
/// "Undefined references from template" check — and every reference maps back
/// to the same source span. Corsa then reports the same template error at each
/// virtual position, which would otherwise surface multiple times (#1389).
/// Severity is part of the key so a genuine error+hint pair on the same span is
/// preserved.
type DiagnosticKey = (std::path::PathBuf, u32, u32, Option<u32>, String, u8);

fn diagnostic_key(diagnostic: &Diagnostic) -> DiagnosticKey {
    (
        diagnostic.file.clone(),
        diagnostic.line,
        diagnostic.column,
        diagnostic.code,
        diagnostic.message.clone(),
        diagnostic.severity,
    )
}

/// Drop exact-duplicate diagnostics while preserving first-seen order, keyed on
/// (file, line, column, code, message, severity).
///
/// A binding with a literal value is checked twice — once by the per-prop
/// `const` annotation, once by the whole-props object literal — and TypeScript
/// renders the same complaint differently at the two sites: the annotation
/// keeps the literal (`Type '123' is not assignable …`) while object-property
/// elaboration widens the fresh literal (`Type 'number' is not assignable …`).
/// `vue-tsc` reports the widened form exactly once, so when both spellings of
/// one complaint land on the same span, only the widened one survives (#4966).
pub(in crate::batch::executor) fn dedup_diagnostics(
    diagnostics: Vec<Diagnostic>,
) -> Vec<Diagnostic> {
    let mut seen: FxHashSet<DiagnosticKey> = FxHashSet::default();
    let mut deduped = Vec::with_capacity(diagnostics.len());
    for diagnostic in diagnostics {
        if seen.insert(diagnostic_key(&diagnostic)) {
            deduped.push(diagnostic);
        }
    }
    deduped.retain(|diagnostic| {
        widen_leading_literal_type(&diagnostic.message).is_none_or(|widened| {
            let mut key = diagnostic_key(diagnostic);
            key.4 = widened;
            !seen.contains(&key)
        })
    });
    deduped
}

/// The message with its leading `Type '<literal>'` widened to the literal's
/// primitive, or `None` when the message does not start with a literal type.
fn widen_leading_literal_type(message: &str) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::dedup_diagnostics;
    use crate::batch::{Diagnostic, SfcBlockType};
    use std::path::PathBuf;

    /// Distinct diagnostics on the same span (different code or message, or a
    /// genuine error+hint pair) must survive deduplication.
    #[test]
    fn dedup_preserves_distinct_diagnostics() {
        let base = Diagnostic {
            file: PathBuf::from("/p/App.vue"),
            line: 4,
            column: 6,
            message: "Cannot find name 'x'.".into(),
            code: Some(2304),
            severity: 1,
            block_type: Some(SfcBlockType::Template),
        };
        let duplicate = base.clone();
        let different_code = Diagnostic {
            code: Some(2322),
            ..base.clone()
        };
        let different_message = Diagnostic {
            message: "Cannot find name 'y'.".into(),
            ..base.clone()
        };
        let different_severity = Diagnostic {
            severity: 4,
            ..base.clone()
        };

        let deduped = dedup_diagnostics(vec![
            base.clone(),
            duplicate,
            different_code,
            different_message,
            different_severity,
        ]);

        assert_eq!(deduped.len(), 4, "{deduped:#?}");
    }

    fn type_mismatch(message: &str) -> Diagnostic {
        Diagnostic {
            file: PathBuf::from("/p/App.vue"),
            line: 11,
            column: 19,
            message: message.into(),
            code: Some(2322),
            severity: 1,
            block_type: Some(SfcBlockType::Template),
        }
    }

    /// One binding, two renderings of one complaint: the widened spelling is
    /// what `vue-tsc` reports, so it is the one that survives (#4966).
    #[test]
    fn literal_rendering_collapses_into_its_widened_twin() {
        for (literal, widened) in [
            (
                "Type '123' is not assignable to type 'boolean | undefined'.",
                "Type 'number' is not assignable to type 'boolean | undefined'.",
            ),
            (
                "Type '\"bad\"' is not assignable to type 'boolean | undefined'.",
                "Type 'string' is not assignable to type 'boolean | undefined'.",
            ),
            (
                "Type 'true' is not assignable to type 'number'.",
                "Type 'boolean' is not assignable to type 'number'.",
            ),
            (
                "Type '12n' is not assignable to type 'number'.",
                "Type 'bigint' is not assignable to type 'number'.",
            ),
        ] {
            let deduped = dedup_diagnostics(vec![type_mismatch(literal), type_mismatch(widened)]);
            let messages: Vec<_> = deduped
                .iter()
                .map(|diagnostic| diagnostic.message.as_str())
                .collect();
            assert_eq!(messages, [widened], "literal spelling: {literal}");
        }
    }

    /// Without the widened twin on the same span the literal rendering is the
    /// only report of the mismatch and must survive.
    #[test]
    fn literal_rendering_without_a_widened_twin_survives() {
        let lone =
            type_mismatch("Type '\"nope\"' is not assignable to type 'Booleanish | undefined'.");
        let different_target = type_mismatch("Type 'string' is not assignable to type 'number'.");
        let mut different_span =
            type_mismatch("Type 'number' is not assignable to type 'Booleanish | undefined'.");
        different_span.column = 30;

        let expected: Vec<_> = [&lone, &different_target, &different_span]
            .into_iter()
            .map(|diagnostic| diagnostic.message.clone())
            .collect();
        let deduped = dedup_diagnostics(vec![lone, different_target, different_span]);
        let messages: Vec<_> = deduped
            .into_iter()
            .map(|diagnostic| diagnostic.message)
            .collect();

        assert_eq!(
            messages, expected,
            "nothing here is the same complaint twice"
        );
    }
}
