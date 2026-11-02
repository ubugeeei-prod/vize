//! Keep generated relative edges on their already-owned TypeScript target.
//!
//! This pass admits only ordinary batch SFC projections. Alias, package, TSX,
//! JavaScript and star-export projections keep their existing resolution path.

use rayon::prelude::*;
use vize_carton::{String, cstr};

use super::super::{VirtualFile, VirtualProject};

impl VirtualProject {
    pub(super) fn finalize_owned_vue_imports(&mut self) {
        // A suffix can precede the ordinary `.ts` candidate. Failed config
        // reads and JavaScript programs cannot establish ordinary resolution.
        let admit = !self.source_policy.allows_javascript()
            && self
                .load_compiler_options(self.resolved_tsconfig_path().as_deref())
                .is_ok_and(|options| !options.contains_key("moduleSuffixes"));
        let rewrites = self
            .virtual_files_sorted()
            .into_par_iter()
            .filter_map(|file| {
                if file.source_map.sfc_map.is_none()
                    || self
                        .source_artifacts
                        .get(&file.original_path)
                        .is_some_and(|owner| owner.star_projection.is_some())
                {
                    return None;
                }
                let pristine = self.pre_rewrite_code.get(&file.virtual_path)?;
                let rewritten = file
                    .source_map
                    .import_map
                    .project_owned_relative_vue_imports(pristine, &file.content, |specifier| {
                        admit
                            .then(|| self.owned_relative_vue_specifier(file, specifier))
                            .flatten()
                            .or_else(|| {
                                self.rewriter
                                    .rewrite_module_specifier_with_missing_vue_policy(
                                        specifier,
                                        file.original_path.parent(),
                                        true,
                                        true,
                                    )
                            })
                    })?;
                Some((file.virtual_path.clone(), rewritten))
            })
            .collect::<Vec<_>>();
        for (path, rewritten) in rewrites {
            let Some(file) = self.virtual_files.get_mut(&path) else {
                continue;
            };
            if file.content != rewritten.code {
                self.incremental_materialized_candidates.insert(path);
            }
            file.content = rewritten.code;
            file.source_map.import_map = rewritten.source_map;
        }
    }

    fn owned_relative_vue_specifier(
        &self,
        importer: &VirtualFile,
        specifier: &str,
    ) -> Option<String> {
        if !(specifier.starts_with("./") || specifier.starts_with("../"))
            || !specifier.ends_with(".vue")
        {
            return None;
        }
        let source = vize_carton::path::canonicalize_non_verbatim(
            &importer.original_path.parent()?.join(specifier),
        );
        let target = self.find_by_original(&source)?;
        if target.source_map.sfc_map.is_none() || target.original_path != source {
            return None;
        }
        let owner = self.source_artifacts.get(&source)?;
        if !owner.virtual_paths.contains(&target.virtual_path) {
            return None;
        }
        // Both old `.vue.ts` and proposed `.vue.js` use this exact `.ts`
        // candidate under ordinary TypeScript extension substitution. Relative
        // imports cannot be intercepted by paths aliases or package export keys.
        let old = cstr!("{specifier}.ts");
        let resolved = super::super::tsconfig_paths::normalize_path_lexically(
            &importer.virtual_path.parent()?.join(old.as_str()),
        );
        if resolved != target.virtual_path
            || self
                .virtual_files
                .get(&resolved)
                .is_none_or(|file| file.original_path != source)
            || resolved
                .extension()
                .is_none_or(|extension| extension != "ts")
        {
            return None;
        }
        Some(cstr!("{specifier}.js"))
    }
}

#[cfg(test)]
mod tests;
