//! Diagnostic offset adjustment for extracted template lint results.

use super::super::config::LintResult;
use crate::context::offset_diagnostic;

pub(crate) fn offset_result(result: &mut LintResult, byte_offset: u32) {
    if byte_offset == 0 {
        return;
    }

    for diag in &mut result.diagnostics {
        offset_diagnostic(diag, byte_offset);
    }
}
