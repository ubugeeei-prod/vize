use std::path::PathBuf;

use oxc_span::SourceType;
use tower_lsp::lsp_types::Url;
use vize_canon::ImportRewriter;

pub(super) fn collect_art_vue_dependency_paths(uri: &Url, code: &str) -> Vec<PathBuf> {
    let Ok(source_path) = uri.to_file_path() else {
        return Vec::new();
    };
    let Some(source_dir) = source_path.parent() else {
        return Vec::new();
    };

    let rewriter = ImportRewriter::new();
    let mut dependencies = Vec::new();
    let mut seen = std::collections::HashSet::<PathBuf>::new();
    for specifier in rewriter.collect_relative_vue_specifiers(code, SourceType::ts(), None) {
        let path = source_dir.join(specifier.as_str());
        let key = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        if seen.insert(key) {
            dependencies.push(path);
        }
    }
    dependencies
}
