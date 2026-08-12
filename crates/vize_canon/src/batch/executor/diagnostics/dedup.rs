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
    deduped
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
}
