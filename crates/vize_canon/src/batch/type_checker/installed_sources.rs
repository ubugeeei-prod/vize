//! Batch checker construction and explicit installed-source lifetime ownership.

use std::path::Path;

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
        let installed = paths.iter().any(|path| {
            if path.is_absolute() {
                is_installed_source(path)
            } else {
                is_installed_source(&self.project.project_root().join(path))
            }
        });
        if !installed {
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
        let storage = tempfile::Builder::new()
            .prefix("batch-installed-")
            .tempdir_in(base)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(storage.path(), std::fs::Permissions::from_mode(0o700))?;
        }
        self.project.scope_batch_namespace(storage.path());
        self.owned_storage = Some(storage);
        Ok(())
    }
}

fn is_installed_source(path: &Path) -> bool {
    contains_node_modules(path)
        || contains_node_modules(&vize_carton::path::canonicalize_non_verbatim(path))
}

fn contains_node_modules(path: &Path) -> bool {
    path.components().any(|part| {
        #[cfg(windows)]
        {
            part.as_os_str()
                .to_str()
                .is_some_and(|value| value.eq_ignore_ascii_case("node_modules"))
        }
        #[cfg(not(windows))]
        {
            part.as_os_str() == "node_modules"
        }
    })
}

#[cfg(test)]
mod tests;
