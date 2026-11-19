use std::path::{Path, PathBuf};

use vize_carton::FxHashMap;

use super::resolution::resolve_dependency_listed;
use super::{
    resolve_dependency, resolve_dependency_with_inputs, resolve_generated_dependency,
    resolve_generated_dependency_with_inputs,
};
use crate::batch::virtual_project::passthrough::PassthroughProbes;
use crate::batch::virtual_project::{VirtualFile, VirtualProject};

#[expect(clippy::disallowed_types, reason = "dependency API uses std String")]
impl VirtualProject {
    pub(super) fn may_have_authored_vue_ts_specifiers(&self) -> bool {
        self.source_artifacts
            .values()
            .any(|artifacts| !artifacts.authored_vue_ts_specifiers.is_empty())
    }

    pub(super) fn source_has_authored_vue_ts_specifiers(&self, source: &Path) -> bool {
        self.source_artifacts
            .get(source)
            .is_some_and(|artifacts| !artifacts.authored_vue_ts_specifiers.is_empty())
    }

    fn source_authored_vue_ts_specifier(
        &self,
        source: &Path,
        specifier: &str,
        may_have_authored_vue_ts: bool,
    ) -> bool {
        if !may_have_authored_vue_ts {
            return false;
        }
        let specifier = specifier
            .strip_suffix(crate::batch::AUTHORED_VUE_TS_ALIAS_SENTINEL)
            .unwrap_or(specifier);
        self.source_artifacts
            .get(source)
            .is_some_and(|artifacts| artifacts.authored_vue_ts_specifiers.contains(specifier))
    }

    pub(super) fn resolve_source_dependency(
        &self,
        source: &Path,
        generated_vue: bool,
        specifier: &str,
        aliases: &[(std::string::String, std::string::String)],
        may_have_authored_vue_ts: bool,
    ) -> Option<PathBuf> {
        if generated_vue
            && !self.source_authored_vue_ts_specifier(source, specifier, may_have_authored_vue_ts)
        {
            return resolve_generated_dependency(
                specifier,
                source.parent()?,
                &self.project_root,
                aliases,
            );
        }
        resolve_dependency(specifier, source.parent()?, &self.project_root, aliases)
    }

    /// [`Self::resolve_source_dependency`] through a walk's directory listings.
    pub(super) fn resolve_source_dependency_listed(
        &self,
        source: &Path,
        generated_vue: bool,
        specifier: &str,
        aliases: &[(std::string::String, std::string::String)],
        probes: &PassthroughProbes,
        may_have_authored_vue_ts: bool,
    ) -> Option<PathBuf> {
        let generated = generated_vue
            && !self.source_authored_vue_ts_specifier(source, specifier, may_have_authored_vue_ts);
        resolve_dependency_listed(
            specifier,
            source.parent()?,
            &self.project_root,
            aliases,
            (generated, probes),
        )
    }

    pub(in crate::batch::virtual_project) fn resolve_file_dependency(
        &self,
        file: &VirtualFile,
        specifier: &str,
        aliases: &[(std::string::String, std::string::String)],
        may_have_authored_vue_ts: bool,
    ) -> Option<PathBuf> {
        self.resolve_source_dependency(
            &file.original_path,
            file.source_map.sfc_map.is_some(),
            specifier,
            aliases,
            may_have_authored_vue_ts,
        )
    }

    pub(in crate::batch::virtual_project) fn resolve_file_dependency_with_inputs(
        &self,
        file: &VirtualFile,
        specifier: &str,
        aliases: &[(std::string::String, std::string::String)],
        may_have_authored_vue_ts: bool,
    ) -> (Option<PathBuf>, Vec<PathBuf>) {
        let authored_vue_ts = self.source_authored_vue_ts_specifier(
            &file.original_path,
            specifier,
            may_have_authored_vue_ts,
        );
        if file.source_map.sfc_map.is_some() && !authored_vue_ts {
            return resolve_generated_dependency_with_inputs(
                specifier,
                file.original_path.parent().unwrap_or(&file.original_path),
                &self.project_root,
                aliases,
            );
        }
        resolve_dependency_with_inputs(
            specifier,
            file.original_path.parent().unwrap_or(&file.original_path),
            &self.project_root,
            aliases,
        )
    }
}

pub(super) fn canonical_importer(known: &FxHashMap<PathBuf, PathBuf>, importer: &Path) -> PathBuf {
    known
        .get(importer)
        .cloned()
        .unwrap_or_else(|| vize_carton::path::canonicalize_non_verbatim(importer))
}
