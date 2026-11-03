//! Ignore directives for virtual ranges no span maps, as Vue's mapper does.

use super::protocol::{
    ContentMapperDiagnosticDirective, ContentMapperDiagnosticDirectives, ContentMapperSpan,
    ContentMapperTransform, DIRECTIVE_POLICY_IGNORE,
};

impl ContentMapperTransform {
    /// Like Vue's mapper, ignore every virtual range no span covers:
    /// TypeScript otherwise reports scaffolding diagnostics at virtual
    /// positions. Explicit tuples keep their intervals, so expectations are
    /// never swallowed.
    pub(super) fn with_unmapped_ignores(mut self) -> Self {
        self.diagnostic_directives = unmapped_ignores(
            self.diagnostic_directives.take(),
            &self.mappings,
            self.text.len(),
        );
        self
    }
}

pub(super) fn unmapped_ignores(
    directives: Option<ContentMapperDiagnosticDirectives>,
    spans: &[ContentMapperSpan],
    generated_len: usize,
) -> Option<ContentMapperDiagnosticDirectives> {
    let mut mapped = spans
        .iter()
        .filter(|ContentMapperSpan([_, length, ..])| *length > 0)
        .map(|ContentMapperSpan([start, length, ..])| *start..start + length)
        .collect::<Vec<_>>();
    mapped.sort_unstable_by_key(|range| range.start);
    let mut unmapped = Vec::new();
    let mut cursor = 0;
    for range in mapped {
        if cursor < range.start {
            unmapped.push(cursor..range.start);
        }
        cursor = cursor.max(range.end);
    }
    if cursor < generated_len {
        unmapped.push(cursor..generated_len);
    }
    let mut covered = directives
        .as_ref()
        .into_iter()
        .flat_map(|table| &table.directives)
        .filter_map(|directive| {
            let [_, _, start, end, _, _] = directive.0;
            (start < end).then_some(start..end)
        })
        .collect::<Vec<_>>();
    covered.sort_unstable_by_key(|range| range.start);
    let mut ignores = Vec::new();
    let mut next = 0;
    for range in unmapped {
        let mut start = range.start;
        while let Some(cover) = covered.get(next)
            && cover.end <= start
        {
            next += 1;
        }
        for cover in covered.iter().skip(next) {
            if cover.start >= range.end {
                break;
            }
            if start < cover.start {
                ignores.push(start..cover.start);
            }
            start = start.max(cover.end);
        }
        if start < range.end {
            ignores.push(start..range.end);
        }
    }
    if ignores.is_empty() {
        return directives;
    }
    let mut directives = directives.unwrap_or(ContentMapperDiagnosticDirectives {
        unused_expect_directive_diagnostics: Vec::new(),
        directives: Vec::new(),
    });
    directives
        .directives
        .extend(ignores.into_iter().map(|range| {
            ContentMapperDiagnosticDirective([
                0,
                0,
                range.start,
                range.end,
                DIRECTIVE_POLICY_IGNORE,
                0,
            ])
        }));
    directives
        .directives
        .sort_by_key(|directive| directive.0[2]);
    Some(directives)
}
