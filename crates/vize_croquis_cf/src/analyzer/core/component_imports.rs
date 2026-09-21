//! Component identity through a file's own imports.

use super::CrossFileAnalyzer;
use super::paths::component_names_match;
use crate::registry::FileId;
use std::path::Path;

impl CrossFileAnalyzer {
    /// The file a component tag used in `file` renders, resolved through that
    /// file's own non-type imports and the module graph only.
    ///
    /// Unlike the component-usage edges, this never falls back to a
    /// project-wide name match: a globally registered or auto-imported
    /// component stays unresolved (`None`), so callers that must prove what a
    /// usage renders (the P4-11b composed HTML check) never act on a guess.
    pub fn resolve_imported_component(&self, file: FileId, tag: &str) -> Option<FileId> {
        let entry = self.registry.get(file)?;
        let current_dir = entry.path.parent().map(Path::to_path_buf);
        entry.analysis.scopes.iter().find_map(|scope| {
            let vize_croquis::ScopeData::ExternalModule(data) = scope.data() else {
                return None;
            };
            if data.is_type_only
                || !scope
                    .bindings()
                    .any(|(local, _)| component_names_match(tag, local))
            {
                return None;
            }
            self.resolve_import(data.source.as_str(), current_dir.as_deref())
        })
    }
}
