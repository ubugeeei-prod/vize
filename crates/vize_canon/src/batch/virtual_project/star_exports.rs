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
        let Some(owner) = self.source_artifacts.get(&original) else {
            return false;
        };
        let mut mapped = owner
            .virtual_paths
            .iter()
            .filter_map(|path| self.virtual_files.get(path))
            .filter(|file| file.original_path == original && file.source_map.sfc_map.is_some());
        let Some(target) = mapped.next() else {
            return false;
        };
        if mapped.next().is_some() || !owner.empty_module_exports {
            return false;
        }
        let Some(mirror_dir) = importer.virtual_path.parent() else {
            return false;
        };
        let mirror = super::tsconfig_paths::normalize_path_lexically(&mirror_dir.join(projected));
        if !owner.virtual_paths.contains(&mirror) {
            return false;
        }
        let Some(file) = self.virtual_files.get(&mirror) else {
            return false;
        };
        if file.virtual_path == target.virtual_path && file.original_path == original {
            return true;
        }
        if target
            .virtual_path
            .extension()
            .is_none_or(|ext| ext != "tsx")
        {
            return false;
        }
        let Ok(expected) = super::build::build_tsx_vue_import_shim(
            &self.project_root,
            &self.virtual_root,
            &original,
            &target.virtual_path,
        ) else {
            return false;
        };
        file.virtual_path == expected.virtual_path
            && file.original_path == expected.original_path
            && file.content == expected.content
            && file.source_map.sfc_map.is_none()
            && file.source_map.import_map == expected.source_map.import_map
    }
}

#[cfg(test)]
mod tests;
