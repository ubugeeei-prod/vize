use super::CrossFileAnalyzer;
use std::path::Path;
use vize_croquis::{Analyzer, Croquis};

impl CrossFileAnalyzer {
    pub(super) fn analyze_single_file(&self, source: &str, path: &Path) -> Croquis {
        let mut analyzer = Analyzer::with_options(self.single_file_options);

        // Detect if it's a Vue SFC
        let is_vue = path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("vue"));

        if is_vue {
            // For Vue SFC, we need the script content extracted.
            // The caller should pass just the script content, or use
            // the WASM bindings which properly parse SFC.
            // For cross-file analysis, we treat Vue SFC source as script setup.
            analyzer.analyze_script_setup(source);
        } else {
            analyzer.analyze_script_plain(source);
        }

        analyzer.finish()
    }
}
