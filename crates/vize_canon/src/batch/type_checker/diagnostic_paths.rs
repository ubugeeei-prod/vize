//! Authored files that stay in the real TypeScript program instead of being
//! copied into Vize's virtual mirror.

use std::path::Path;

use super::BatchTypeChecker;

impl BatchTypeChecker {
    /// Diagnose authored program files in place without registering them as
    /// virtual roots. This preserves module identity for generated sources and
    /// lets TypeScript own alias and absolute-import resolution.
    pub fn set_diagnostic_paths<'a>(&mut self, paths: impl IntoIterator<Item = &'a Path>) {
        self.project.set_diagnostic_paths(paths);
    }
}
