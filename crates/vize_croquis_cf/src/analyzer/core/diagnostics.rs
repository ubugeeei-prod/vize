use crate::diagnostics::{CrossFileDiagnostic, DiagnosticSeverity};
use crate::registry::FileId;
use vize_carton::FxHashMap;

pub(super) fn dedupe_diagnostics(diagnostics: &mut Vec<CrossFileDiagnostic>) {
    let mut seen: FxHashMap<(&'static str, FileId, u32), usize> = FxHashMap::default();
    let mut deduped: Vec<CrossFileDiagnostic> = Vec::with_capacity(diagnostics.len());

    for diagnostic in diagnostics.drain(..) {
        let key = (
            diagnostic.code(),
            diagnostic.primary_file,
            diagnostic.primary_offset,
        );

        if let Some(index) = seen.get(&key).copied() {
            merge_duplicate_diagnostic(&mut deduped[index], diagnostic);
        } else {
            seen.insert(key, deduped.len());
            deduped.push(diagnostic);
        }
    }

    *diagnostics = deduped;
}

fn merge_duplicate_diagnostic(existing: &mut CrossFileDiagnostic, incoming: CrossFileDiagnostic) {
    if is_more_severe(incoming.severity, existing.severity) {
        existing.severity = incoming.severity;
    }

    if existing.suggestion.is_none() {
        existing.suggestion = incoming.suggestion.clone();
    }

    for related in incoming.related_files {
        if !existing.related_files.iter().any(|entry| entry == &related) {
            existing.related_files.push(related);
        }
    }
}

pub(super) fn sort_diagnostics(diagnostics: &mut [CrossFileDiagnostic]) {
    diagnostics.sort_by(|left, right| {
        left.primary_file
            .as_u32()
            .cmp(&right.primary_file.as_u32())
            .then_with(|| left.primary_offset.cmp(&right.primary_offset))
            .then_with(|| severity_order(left.severity).cmp(&severity_order(right.severity)))
            .then_with(|| left.code().cmp(right.code()))
    });
}

fn is_more_severe(candidate: DiagnosticSeverity, current: DiagnosticSeverity) -> bool {
    severity_order(candidate) < severity_order(current)
}

fn severity_order(severity: DiagnosticSeverity) -> u8 {
    severity as u8
}
