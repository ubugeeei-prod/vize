use super::super::collect_virtual::{CorsaFinished, finished_from_phased_lsp};
use vize_canon::corsa_bridge::PhasedLspDiagnostic;
use vize_l0::{SmallVec, line_index::LineBreaks};

pub(super) fn finish(code: &str, diagnostics: Vec<PhasedLspDiagnostic>) -> Vec<CorsaFinished> {
    if diagnostics.len() < 2 {
        return diagnostics
            .into_iter()
            .filter_map(|row| finished_from_phased_lsp(code, row, 0))
            .collect();
    }

    let mut starts: SmallVec<[usize; 16]> = vize_l0::smallvec![0];
    let mut remaining = LineBreaks::Lsp.line_starts(code).skip(1);
    diagnostics
        .into_iter()
        .filter_map(|mut row| {
            // Decode from the first line touched by either endpoint. This also
            // preserves the ordinary decoder's behavior for reversed or invalid ranges.
            let range = &mut row.diagnostic.range;
            let line = range.start.line.min(range.end.line);
            let index = line as usize;
            if index >= starts.len() {
                starts.extend(remaining.by_ref().take(index - (starts.len() - 1)));
            }
            let base = *starts.get(index)?;
            range.start.line -= line;
            range.end.line -= line;
            let mut decoded = finished_from_phased_lsp(code.get(base..)?, row, 0)?;
            decoded.start += base;
            decoded.end += base;
            Some(decoded)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use vize_canon::corsa_bridge::DiagnosticPhase;

    #[test]
    fn every_projected_field_matches_for_reversed_and_invalid_endpoints() {
        let code = "a🌸\r\n雪\rb\u{2028}c\u{2029}\n";
        let points = [
            (0, 0),
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 0),
            (1, 1),
            (2, 0),
            (2, 2),
            (2, 4),
            (3, 0),
            (3, 1),
            (4, 0),
            (u32::MAX, u32::MAX),
        ];
        let mut rows = Vec::new();
        for (line, character) in points.into_iter().rev() {
            for (end_line, end_character) in points {
                let diagnostic = serde_json::from_value(serde_json::json!({
                    "range": {"start": {"line": line, "character": character},
                              "end": {"line": end_line, "character": end_character}},
                    "severity": 2, "code": "TS2339", "source": "ts", "message": "missing"
                }))
                .unwrap();
                rows.push(PhasedLspDiagnostic {
                    phase: DiagnosticPhase::Semantic,
                    diagnostic,
                });
            }
        }
        for count in [0, 1, 2, rows.len()] {
            let batch = &rows[..count];
            let expected: Vec<_> = batch
                .iter()
                .cloned()
                .filter_map(|row| finished_from_phased_lsp(code, row, 0))
                .collect();
            assert_eq!(finish(code, batch.to_vec()), expected);
        }
    }
}
