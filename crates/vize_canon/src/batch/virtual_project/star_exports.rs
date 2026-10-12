//! Same-owner baseline for graph-dependent star edits without another parse.

use crate::batch::import_rewriter::{EmptyVueStar, ImportSourceMap, RewriteResult};
use vize_carton::String;

pub(super) struct StarProjection {
    pub(super) code: String,
    pub(super) source_map: ImportSourceMap,
    pub(super) stars: Vec<EmptyVueStar>,
}

impl StarProjection {
    pub(super) fn from_rewrite(result: &RewriteResult, stars: Vec<EmptyVueStar>) -> Option<Self> {
        (!stars.is_empty()).then(|| Self {
            code: result.code.clone(),
            source_map: result.source_map.clone(),
            stars,
        })
    }
}

impl super::VirtualProject {
    /// Only a direct authored edge and its actual same-owner mirror can erase
    /// a star. Alias, package and extensionless routes remain conservative.
    pub(super) fn known_empty_vue_target(
        &self,
        importer: &super::VirtualFile,
        specifier: &str,
        projected: &str,
    ) -> bool {
        if !(specifier.starts_with("./") || specifier.starts_with("../"))
            || std::path::Path::new(specifier)
                .extension()
                .is_none_or(|ext| ext != "vue")
            || projected.strip_suffix(".ts") != Some(specifier)
        {
            return false;
        }
        let Some(source_dir) = importer.original_path.parent() else {
            return false;
        };
        let original = vize_carton::path::canonicalize_non_verbatim(&source_dir.join(specifier));
        let Some(target) = self.find_by_original(&original) else {
            return false;
        };
        if target.source_map.sfc_map.is_none()
            || target
                .original_path
                .extension()
                .is_none_or(|ext| ext != "vue")
        {
            return false;
        }
        let Some(owner) = self.source_artifacts.get(&target.original_path) else {
            return false;
        };
        let Some(mirror_dir) = importer.virtual_path.parent() else {
            return false;
        };
        let mirror = super::tsconfig_paths::normalize_path_lexically(&mirror_dir.join(projected));
        owner.empty_module_exports
            && owner.virtual_paths.contains(&mirror)
            && self
                .virtual_files
                .get(&mirror)
                .is_some_and(|file| file.original_path == target.original_path)
    }
}

#[cfg(test)]
mod tests;
