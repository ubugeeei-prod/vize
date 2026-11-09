use std::path::{Path, PathBuf};

use super::resolution::resolve_dependency_listed;
use super::{
    resolve_dependency, resolve_dependency_with_inputs, resolve_generated_dependency,
    resolve_generated_dependency_with_inputs,
};
use crate::batch::virtual_project::passthrough::PassthroughProbes;
use crate::batch::virtual_project::{VirtualFile, VirtualProject};

#[expect(clippy::disallowed_types, reason = "dependency API uses std String")]
impl VirtualProject {
    fn source_authored_vue_ts_specifier(&self, source: &Path, specifier: &str) -> bool {
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
    ) -> Option<PathBuf> {
        if generated_vue && !self.source_authored_vue_ts_specifier(source, specifier) {
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
    ) -> Option<PathBuf> {
        let generated = generated_vue && !self.source_authored_vue_ts_specifier(source, specifier);
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
    ) -> Option<PathBuf> {
        self.resolve_source_dependency(
            &file.original_path,
            file.source_map.sfc_map.is_some(),
            specifier,
            aliases,
        )
    }

    pub(in crate::batch::virtual_project) fn resolve_file_dependency_with_inputs(
        &self,
        file: &VirtualFile,
        specifier: &str,
        aliases: &[(std::string::String, std::string::String)],
    ) -> (Option<PathBuf>, Vec<PathBuf>) {
        let authored_vue_ts = self.source_authored_vue_ts_specifier(&file.original_path, specifier);
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
