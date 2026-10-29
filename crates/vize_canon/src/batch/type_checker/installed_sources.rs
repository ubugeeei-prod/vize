//! Batch checker construction and explicit installed-source lifetime ownership.

use std::path::{Path, PathBuf};

use rayon::prelude::*;

mod persistent;
pub(super) use persistent::PersistentStorageOwner;

use super::{
    BatchTypeChecker, BatchTypeCheckerOptions, CorsaExecutor, CorsaResult, IncrementalPaths,
    VirtualProject,
};

impl BatchTypeChecker {
    /// Create a new batch type checker.
    pub fn new(project_root: &Path) -> CorsaResult<Self> {
        Self::with_options(project_root, BatchTypeCheckerOptions::default())
    }

    /// Create a new batch type checker with explicit options.
    pub fn with_options(
        project_root: &Path,
        options: BatchTypeCheckerOptions,
    ) -> CorsaResult<Self> {
        Self::with_options_and_corsa_path(project_root, options, None)
    }

    /// Create a new batch type checker with options and an optional Corsa path.
    pub fn with_options_and_corsa_path(
        project_root: &Path,
        options: BatchTypeCheckerOptions,
        corsa_path: Option<&Path>,
    ) -> CorsaResult<Self> {
        let project = VirtualProject::new(project_root)?;
        let mut project = project;
        project.set_tsconfig_path(options.tsconfig_path);
        project.set_virtual_ts_options(options.virtual_ts_options);
        let executor = CorsaExecutor::with_corsa_path(project.project_root(), corsa_path)?;

        Ok(Self {
            project,
            executor,
            scanned: false,
            server_count: None,
            incremental_paths: IncrementalPaths::new(),
            owned_storage: None,
            persistent_storage_owner: None,
            _persistent_storage_leases: None,
        })
    }
}

impl BatchTypeChecker {
    pub(super) fn scope_initial_installed_sources(
        &mut self,
        paths: &[std::path::PathBuf],
    ) -> CorsaResult<()> {
        if self.scanned || self.owned_storage.is_some() {
            return Ok(());
        }
        let project_root = self.project.project_root();
        let installed = paths.par_iter().any(|path| {
            if path.is_absolute() {
                is_installed_source(project_root, path)
            } else {
                is_installed_source(project_root, &project_root.join(path))
            }
        });
        if self.scope_persistent_initial_storage(installed)? || !installed {
            return Ok(());
        }
        // Scope before registration computes any virtual path. Package-route
        // bindings may already exist, but they have no materialized artifacts.
        if self.project.file_count() != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Installed source roots require a fresh checker before registration",
            )
            .into());
        }
        let base = self
            .project
            .virtual_root()
            .parent()
            .ok_or_else(|| std::io::Error::other("Batch virtual root has no storage parent"))?;
        std::fs::create_dir_all(base)?;
        let key = self.project.installed_storage_key();
        let mut storage = super::installed_storage::InstalledStorage::claim(base, &key)?;
        storage.reset(&self.project.batch_build_info_dir(storage.path()))?;
        self.project.scope_batch_namespace(storage.path());
        self.owned_storage = Some(storage);
        Ok(())
    }
}

/// Whether `path` is an installed file of this project: a `node_modules`
/// segment lies below the project root in its logical spelling or in its
/// canonical one.
///
/// Private storage exists so an owned write below the virtual root's own
/// `node_modules/` cannot meet a package link that a previous command left in
/// the persistent namespace (#8274). A file outside the project root never
/// lands there: it mirrors into `__vize_external__`. A pnpm store file, or a
/// declaration that a shared tsconfig lists from a hoisted workspace root,
/// therefore keeps the persistent namespace, with its skipped rewrites and its
/// incremental build info.
fn is_installed_source(project_root: &Path, path: &Path) -> bool {
    installed_below_root(project_root, path)
        || installed_below_root(
            project_root,
            &vize_carton::path::canonicalize_non_verbatim(path),
        )
}

/// Every `node_modules` segment is tried, so a project that itself lives
/// below a `node_modules` directory still owns the files below its own one.
/// The directory before a segment exists in either spelling. Canonicalizing
/// only that much keeps a logical link root (`node_modules/selected` ->
/// elsewhere) below its project root.
fn installed_below_root(project_root: &Path, path: &Path) -> bool {
    path.components()
        .enumerate()
        .filter(|(_, part)| is_node_modules_component(part.as_os_str()))
        .any(|(installed_at, _)| {
            let prefix = path.components().take(installed_at).collect::<PathBuf>();
            vize_carton::path::canonicalize_non_verbatim(&prefix).starts_with(project_root)
        })
}

#[cfg(test)]
fn contains_node_modules(path: &Path) -> bool {
    path.components()
        .any(|part| is_node_modules_component(part.as_os_str()))
}

fn is_node_modules_component(part: &std::ffi::OsStr) -> bool {
    #[cfg(windows)]
    {
        part.to_str()
            .is_some_and(|value| value.eq_ignore_ascii_case("node_modules"))
    }
    #[cfg(not(windows))]
    {
        part == "node_modules"
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod persistent_tests;
