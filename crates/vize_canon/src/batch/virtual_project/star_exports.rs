//! Same-owner baseline for graph-dependent star edits without another parse.

use crate::batch::import_rewriter::{EmptyVueStar, ImportSourceMap, RewriteResult, VueStarTarget};
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
    /// Only a direct authored edge and its actual same-owner mirror can narrow
    /// a star. Alias, package and extensionless routes remain conservative.
    pub(super) fn known_vue_star_target(
        &self,
        importer: &super::VirtualFile,
        specifier: &str,
        projected: &str,
    ) -> Option<VueStarTarget<'_>> {
        if !(specifier.starts_with("./") || specifier.starts_with("../"))
            || std::path::Path::new(specifier)
                .extension()
                .is_none_or(|ext| ext != "vue")
            || projected.strip_suffix(".ts") != Some(specifier)
        {
            return None;
        }
        let source_dir = importer.original_path.parent()?;
        let original = vize_carton::path::canonicalize_non_verbatim(&source_dir.join(specifier));
        let owner = self.source_artifacts.get(&original)?;
        let mut mapped = owner
            .virtual_paths
            .iter()
            .filter_map(|path| self.virtual_files.get(path))
            .filter(|file| file.original_path == original && file.source_map.sfc_map.is_some());
        let target = mapped.next()?;
        if mapped.next().is_some() {
            return None;
        }
        let star_target = if owner.empty_module_exports {
            VueStarTarget::Empty
        } else if let Some(names) = &owner.module_exports {
            VueStarTarget::Names(names)
        } else {
            return None;
        };
        let mirror_dir = importer.virtual_path.parent()?;
        let mirror = super::tsconfig_paths::normalize_path_lexically(&mirror_dir.join(projected));
        if !owner.virtual_paths.contains(&mirror) {
            return None;
        }
        let file = self.virtual_files.get(&mirror)?;
        if file.virtual_path == target.virtual_path && file.original_path == original {
            return Some(star_target);
        }
        if target
            .virtual_path
            .extension()
            .is_none_or(|ext| ext != "tsx")
        {
            return None;
        }
        let Ok(expected) = super::build::build_tsx_vue_import_shim(
            &self.project_root,
            &self.virtual_root,
            &original,
            &target.virtual_path,
        ) else {
            return None;
        };
        (file.virtual_path == expected.virtual_path
            && file.original_path == expected.original_path
            && file.content == expected.content
            && file.source_map.sfc_map.is_none()
            && file.source_map.import_map == expected.source_map.import_map)
            .then_some(star_target)
    }
}

#[cfg(test)]
mod tests;
