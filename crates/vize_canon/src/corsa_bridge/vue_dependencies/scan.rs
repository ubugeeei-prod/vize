//! Reusable dependency-edge discovery for persistent editor compilations.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;

use super::{normalize_path, parent_dir, resolve_relative_script_import, source_type_for_path};
use crate::batch::ImportRewriter;
use crate::corsa_bridge::vue_dependency_specifiers::collect_relative_ts_specifiers;

/// Relative dependency paths discovered with the same resolver used by Corsa sync.
pub struct CorsaRelativeDependencies {
    pub vue: Vec<PathBuf>,
    pub scripts: Vec<CorsaScriptDependency>,
}

/// A resolved script dependency and the source kind used for its next scan.
pub struct CorsaScriptDependency {
    pub path: PathBuf,
    pub source_type: SourceType,
}

/// Resolve one generated Vue or authored script document's dependency edges.
pub fn collect_corsa_relative_dependencies(
    source_path: &Path,
    code: &str,
    source_type: SourceType,
) -> CorsaRelativeDependencies {
    let directory = parent_dir(source_path);
    let rewriter = ImportRewriter::new();
    let vue = rewriter
        .collect_relative_vue_specifiers(code, source_type)
        .into_iter()
        .map(|specifier| normalize_path(&directory.join(specifier.as_str())))
        .collect();
    let scripts = collect_relative_ts_specifiers(code, source_type)
        .into_iter()
        .filter_map(|specifier| {
            let path = resolve_relative_script_import(&directory, specifier.as_str())?;
            Some(CorsaScriptDependency {
                source_type: source_type_for_path(&path),
                path,
            })
        })
        .collect();
    CorsaRelativeDependencies { vue, scripts }
}
