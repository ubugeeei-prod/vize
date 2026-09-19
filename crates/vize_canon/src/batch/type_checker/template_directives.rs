//! Apply the shared authored-node policy once per registered source snapshot.

use crate::batch::{Diagnostic, SfcBlockType, VirtualProject};
use crate::template_diagnostic_directives::{
    TemplateDiagnosticDirectives, UNUSED_EXPECT_ERROR_CODE, UNUSED_EXPECT_ERROR_MESSAGE,
};
use vize_carton::{FxHashMap, line_index::LineIndex};

pub(super) fn apply(diagnostics: &mut Vec<Diagnostic>, project: &VirtualProject) {
    let mut by_path = FxHashMap::<_, Vec<_>>::default();
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if diagnostic.code.is_some() {
            by_path
                .entry(diagnostic.file.clone())
                .or_default()
                .push(index);
        }
    }
    let mut suppressed = vec![false; diagnostics.len()];
    let mut unused = Vec::new();
    for file in project.virtual_files_sorted() {
        if file
            .original_path
            .extension()
            .is_none_or(|extension| extension != "vue")
        {
            continue;
        }
        let Some(source) = project.original_content_for_virtual(&file.virtual_path) else {
            continue;
        };
        let mut directives = TemplateDiagnosticDirectives::for_sfc(source);
        if directives.directives().is_empty() {
            continue;
        }
        let lines = LineIndex::new(source);
        for &index in by_path.get(&file.original_path).into_iter().flatten() {
            let diagnostic = &diagnostics[index];
            if let Some(offset) = lines.line_col_to_offset(diagnostic.line, diagnostic.column) {
                suppressed[index] = directives.suppresses(offset);
            }
        }
        for range in directives.unused_expectations() {
            let (line, column) = lines.line_col(range.start);
            unused.push(Diagnostic {
                file: file.original_path.clone(),
                line,
                column,
                message: UNUSED_EXPECT_ERROR_MESSAGE.into(),
                code: Some(UNUSED_EXPECT_ERROR_CODE),
                severity: 1,
                block_type: Some(SfcBlockType::Template),
            });
        }
    }
    let mut index = 0;
    diagnostics.retain(|_| {
        let retain = !suppressed[index];
        index += 1;
        retain
    });
    diagnostics.extend(unused);
}
