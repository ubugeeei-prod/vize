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

#[cfg(test)]
mod tests {
    use super::BatchTypeChecker;
    use crate::batch::BatchTypeCheckerOptions;

    #[test]
    fn combined_scan_installs_the_valid_canonical_diagnostic_snapshot() {
        let root = tempfile::tempdir().unwrap();
        let binary = root.path().join("unused-tsgo");
        std::fs::write(&binary, "unused fixture executable").unwrap();
        let source_dir = root.path().join("src");
        std::fs::create_dir_all(source_dir.join("nested")).unwrap();
        let source = source_dir.join("app.ts");
        std::fs::write(&source, "export const value = 1;\n").unwrap();
        let logical = source_dir.join("nested/../app.ts");
        let missing = source_dir.join("missing.ts");
        let canonical = vize_carton::path::canonicalize_non_verbatim(&source);
        let mut checker = BatchTypeChecker::with_options_and_corsa_path(
            root.path(),
            BatchTypeCheckerOptions::default(),
            Some(binary.as_path()),
        )
        .unwrap();

        checker
            .scan_paths_with_diagnostics(&[logical, missing])
            .unwrap();

        assert_eq!(checker.project.diagnostic_paths_sorted(), vec![canonical]);
    }
}
