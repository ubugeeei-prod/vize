use super::DiagnosticMapper;
use std::path::Path;

impl DiagnosticMapper<'_> {
    /// Only the generated reachability assertion is downgraded. Ordinary
    /// assignability failures inside an arm remain errors.
    pub(in crate::batch::executor) fn is_unreachable_pattern(
        &mut self,
        path: &Path,
        line: u32,
        column: u32,
        code: Option<u32>,
    ) -> bool {
        if code != Some(2322) {
            return false;
        }
        let Some(file) = self.project.find_by_diagnostic_virtual(path) else {
            return false;
        };
        let Some(offset) = self.virtual_offset(file, line, column) else {
            return false;
        };
        crate::virtual_ts::is_unreachable_pattern_diagnostic(&file.content, offset as usize, code)
    }
}
