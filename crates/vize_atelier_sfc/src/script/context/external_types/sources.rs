use std::path::Path;
use vize_carton::{FxHashSet, String};

use super::{FileTypeSummary, extract_file_summary, extract_script_summary, path_key};
use crate::script::{ScriptCompileContext, TypeSourceSnapshot};

impl ScriptCompileContext {
    /// Resolve compatibility props against the same immutable source view used
    /// by the public type world. Disk-global summary caches are bypassed.
    pub fn collect_imported_types_from_path_with_sources(
        &mut self,
        source: &str,
        filename: &str,
        is_ts: bool,
        sources: &TypeSourceSnapshot,
    ) {
        if !is_ts {
            return;
        }
        let mut root = FileTypeSummary::default();
        extract_script_summary(source, &mut root, false);
        let base = super::super::source_snapshot::source_path(Path::new(filename));
        let mut visited = FxHashSet::default();
        for specifier in root.specifiers {
            self.collect_types_with_sources(&specifier, &base, &mut visited, sources);
        }
    }

    fn collect_types_with_sources(
        &mut self,
        specifier: &str,
        current: &Path,
        visited: &mut FxHashSet<String>,
        sources: &TypeSourceSnapshot,
    ) {
        let Some(path) = sources.resolve_import_from_identity(current, specifier) else {
            return;
        };
        if visited.len() >= 512 || !visited.insert(path_key(&path)) {
            return;
        }
        let Some(content) = sources.read_identity(path.clone()) else {
            return;
        };
        let specifiers = {
            let Ok(mut cache) = sources.summaries.lock() else {
                return;
            };
            let summary = cache.entry(path.clone()).or_insert_with(|| {
                let is_vue = path.extension().is_some_and(|ext| ext == "vue");
                let follows_values = path.file_name().is_some_and(|name| {
                    let name = name.to_string_lossy();
                    name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
                });
                extract_file_summary(&content, is_vue, follows_values)
            });
            self.merge_file_summary(summary);
            summary.specifiers.clone()
        };
        for specifier in specifiers {
            self.collect_types_with_sources(&specifier, &path, visited, sources);
        }
    }
}
