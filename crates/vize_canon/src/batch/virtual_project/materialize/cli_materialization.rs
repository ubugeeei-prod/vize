//! Transaction-local CLI build-info authority; no project-global caching.

use std::path::{Path, PathBuf};

use super::{CorsaResult, MaterializeLock, VirtualProject};

/// A completed materialization's exact desired plan. It is consumed only
/// after the executor decides that the CLI route is eligible; session-only
/// checks discard it without hashing or retaining the map during typechecking.
pub(crate) struct CliMaterialization<'a> {
    project: &'a VirtualProject,
    materialize_lock: &'a MaterializeLock,
    links: vize_carton::FxHashMap<PathBuf, PathBuf>,
}

impl<'a> CliMaterialization<'a> {
    // Only the parent materializer can create authority, after all writes and
    // the final custody receipt have succeeded.
    pub(super) fn new(
        project: &'a VirtualProject,
        materialize_lock: &'a MaterializeLock,
        links: vize_carton::FxHashMap<PathBuf, PathBuf>,
    ) -> Self {
        Self {
            project,
            materialize_lock,
            links,
        }
    }

    pub(crate) fn into_build_info(self) -> CliBuildInfo<'a> {
        CliBuildInfo {
            project: self.project,
            _materialize_lock: self.materialize_lock,
            fingerprint: self.project.cli_build_info_topology(self.links),
        }
    }
}

/// Build-info authority bound to this exact project and supplied guard.
/// Borrowing keeps that guard alive; the executor is responsible for acquiring
/// it for this project's root. Only a fingerprint survives into the Corsa check.
pub(crate) struct CliBuildInfo<'a> {
    project: &'a VirtualProject,
    _materialize_lock: &'a MaterializeLock,
    fingerprint: Option<vize_carton::String>,
}

impl CliBuildInfo<'_> {
    pub(crate) fn build_info_path(
        &self,
        project: &VirtualProject,
        config_path: &Path,
    ) -> CorsaResult<PathBuf> {
        if !std::ptr::eq(self.project, project) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "CLI materialization belongs to a different virtual project",
            )
            .into());
        }
        Ok(project.cli_build_info_path_with_topology(config_path, self.fingerprint.as_deref()))
    }
}
