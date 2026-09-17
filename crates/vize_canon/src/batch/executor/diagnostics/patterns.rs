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
        let rest = &file.content[offset as usize..];
        let Some((name, suffix)) = rest.split_once(':') else {
            return false;
        };
        name.starts_with("__vize_match_")
            && name.ends_with("unreachable")
            && suffix.starts_with(" __VizePatterns.Reachable<")
    }
}
