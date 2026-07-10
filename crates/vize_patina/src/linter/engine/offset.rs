//! Diagnostic offset adjustment for extracted template lint results.

use super::super::config::LintResult;

pub(crate) fn offset_result(result: &mut LintResult, byte_offset: u32) {
    if byte_offset == 0 {
        return;
    }

    for diag in &mut result.diagnostics {
        diag.start += byte_offset;
        diag.end += byte_offset;
        for label in &mut diag.labels {
            label.start += byte_offset;
            label.end += byte_offset;
        }
        if let Some(fix) = diag.fix.as_mut() {
            for edit in &mut fix.edits {
                edit.start += byte_offset;
                edit.end += byte_offset;
            }
        }
    }
}
