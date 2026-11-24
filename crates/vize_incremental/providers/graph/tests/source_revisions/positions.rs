//! Independent per-record scan of authored cursor requests on all original revisions.
use super::{BlockView, Span, TemplateRecovery};
use core::ops::ControlFlow;
use vize_resident_graph::RecoveryRecord;

pub(super) fn check(recovery: &TemplateRecovery<'_>, source: &str, block: BlockView<'_>) -> usize {
    let mut source_rows = Vec::new();
    for hole in recovery.holes() {
        source_rows.push(RecoveryRecord::Hole(hole));
    }
    for diagnostic in recovery.diagnostics() {
        source_rows.push(RecoveryRecord::Diagnostic(diagnostic));
        for part in &diagnostic.parts {
            source_rows.push(RecoveryRecord::Part { diagnostic, part });
        }
        if let Some(witness) = diagnostic.witness_chain() {
            for link in witness.links() {
                source_rows.push(RecoveryRecord::Witness { diagnostic, link });
            }
        }
    }
    let base = block.start();
    let text = block.source().text.as_str();
    let mut points = vec![base, base + text.len() as u32];
    for (offset, _) in text.char_indices().step_by((text.len() / 16).max(1)) {
        points.push(base + offset as u32);
    }
    for row in source_rows.iter().take(12).chain(source_rows.last()) {
        let range = row.relative_span();
        points.extend([base + range.start, base + range.end]);
    }
    points.sort_unstable();
    let mut expected = source_rows.iter().filter_map(|row| {
        let relative = row.relative_span();
        let authored = Span::new(base + relative.start, base + relative.end);
        let original = &source[authored.start as usize..authored.end as usize];
        let hits: Vec<_> = points
            .iter()
            .copied()
            .filter(|&p| {
                (authored.start <= p && p < authored.end)
                    || (authored.start == authored.end && p == authored.start)
            })
            .collect();
        (!hits.is_empty()).then_some((*row, authored, original, hits))
    });
    let mut matches = 0;
    let result = recovery.visit_positions(&points, |row| {
        let (original, span, text, hits) = expected.next().expect("unexpected cursor result");
        assert_eq!(row.record, original);
        assert_eq!(row.authored, span);
        assert_eq!(row.text, text);
        assert_eq!(points[row.positions], hits);
        matches += hits.len();
        ControlFlow::<()>::Continue(())
    });
    assert_eq!(result, Ok(ControlFlow::Continue(())));
    assert!(expected.next().is_none());
    matches
}
