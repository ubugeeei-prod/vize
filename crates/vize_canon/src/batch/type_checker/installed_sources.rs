//! Batch checker construction and explicit installed-source lifetime ownership.

use std::path::Path;

use rayon::prelude::*;

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
        let installed = paths.par_iter().any(|path| {
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
mod tests {
    #![expect(
        clippy::disallowed_macros,
        reason = "fixture assertions use std formatting"
    )]
    #![expect(
        clippy::disallowed_methods,
        reason = "fixture assertions use std strings"
    )]
    #![expect(
        clippy::disallowed_types,
        reason = "fixture assertions use std strings"
    )]
    //! Private batch storage must remain alive with its owner and disappear on drop.

    use super::{BatchTypeChecker, BatchTypeCheckerOptions, contains_node_modules};
    use std::path::{Path, PathBuf};

    fn checker(root: &Path) -> BatchTypeChecker {
        let binary = root.join("unused-tsgo");
        std::fs::write(&binary, "unused fixture executable").unwrap();
        BatchTypeChecker::with_options_and_corsa_path(
            root,
            BatchTypeCheckerOptions::default(),
            Some(&binary),
        )
        .unwrap()
    }

    fn source(root: &Path, relative: &str) -> PathBuf {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "export const selected: number = 1\n").unwrap();
        path
    }

    #[test]
    fn only_installed_path_components_select_private_storage() {
        assert!(contains_node_modules(Path::new(
            "node_modules/selected/index.ts"
        )));
        assert!(!contains_node_modules(Path::new(
            "src/my_node_modules/index.ts"
        )));
        assert!(!contains_node_modules(Path::new("src/node_modules.ts")));
        #[cfg(windows)]
        assert!(contains_node_modules(Path::new(
            "NODE_MODULES/selected/index.ts"
        )));
        #[cfg(not(windows))]
        assert!(!contains_node_modules(Path::new(
            "NODE_MODULES/selected/index.ts"
        )));
    }

    #[test]
    fn ordinary_roots_keep_the_existing_persistent_namespace() {
        let root = tempfile::tempdir().unwrap();
        let selected = source(root.path(), "src/index.ts");
        let mut owner = checker(root.path());
        let expected = crate::batch::project_virtual_root(root.path());
        owner.scan_paths(&[selected]).unwrap();
        assert!(owner.owned_storage.is_none());
        assert_eq!(owner.project.virtual_root(), expected);
    }

    #[test]
    fn installed_owner_reuses_private_paths_and_cleans_successful_materialization() {
        let root = tempfile::tempdir().unwrap();
        let selected = source(root.path(), "node_modules/selected/index.ts");
        let raw = std::fs::read(&selected).unwrap();
        let mut owner = checker(root.path());
        owner.scan_paths(std::slice::from_ref(&selected)).unwrap();
        let storage = owner.owned_storage.as_ref().unwrap().path().to_path_buf();
        let namespace = owner.project.virtual_root().to_path_buf();
        assert_eq!(
            namespace
                .ancestors()
                .find(|path| *path == storage.as_path())
                .map(Path::to_path_buf),
            Some(storage.clone())
        );
        assert_ne!(namespace, crate::batch::project_virtual_root(root.path()));
        assert_eq!(namespace.strip_prefix(root.path()).ok(), None);
        assert!(!contains_node_modules(&namespace));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&storage).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        owner.project.materialize().unwrap();
        let virtual_paths = owner
            .project
            .virtual_files_sorted()
            .into_iter()
            .map(|file| file.virtual_path.clone())
            .collect::<Vec<_>>();
        owner.scan_paths(std::slice::from_ref(&selected)).unwrap();
        assert_eq!(owner.project.virtual_root(), namespace);
        assert_eq!(
            owner
                .project
                .virtual_files_sorted()
                .into_iter()
                .map(|file| file.virtual_path.clone())
                .collect::<Vec<_>>(),
            virtual_paths
        );
        assert!(namespace.is_dir());
        drop(owner);
        assert!(!storage.exists());
        assert_eq!(std::fs::read(&selected).unwrap(), raw);
    }

    #[test]
    fn installed_owner_cleans_storage_after_registration_error() {
        let root = tempfile::tempdir().unwrap();
        let invalid = source(root.path(), "node_modules/selected/invalid.ts");
        std::fs::write(&invalid, [0xff]).unwrap();
        let mut owner = checker(root.path());
        assert!(matches!(owner.scan_paths(&[invalid]),
        Err(crate::batch::error::CorsaError::Io(error))
            if error.kind() == std::io::ErrorKind::InvalidData));
        let storage = owner.owned_storage.as_ref().unwrap().path().to_path_buf();
        assert!(storage.is_dir());
        drop(owner);
        assert!(!storage.exists());
    }

    #[test]
    fn installed_owner_cleans_storage_during_unwind() {
        let root = tempfile::tempdir().unwrap();
        let selected = source(root.path(), "node_modules/selected/index.ts");
        let mut storage = None;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut owner = checker(root.path());
            owner.scan_paths(&[selected]).unwrap();
            owner.project.materialize().unwrap();
            storage = Some(owner.owned_storage.as_ref().unwrap().path().to_path_buf());
            panic!("exercise batch owner unwind");
        }));
        assert!(result.is_err());
        assert!(!storage.unwrap().exists());
    }

    #[cfg(unix)]
    #[test]
    fn logical_installed_symlink_roots_still_select_private_storage() {
        let root = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        source(external.path(), "index.ts");
        std::fs::create_dir(root.path().join("node_modules")).unwrap();
        std::os::unix::fs::symlink(external.path(), root.path().join("node_modules/selected"))
            .unwrap();
        let mut owner = checker(root.path());
        owner
            .scan_paths(&[root.path().join("node_modules/selected/index.ts")])
            .unwrap();
        let storage = owner.owned_storage.as_ref().unwrap().path().to_path_buf();
        drop(owner);
        assert!(!storage.exists());
        assert_eq!(
            std::fs::read_link(root.path().join("node_modules/selected")).unwrap(),
            external.path()
        );
    }

    #[test]
    fn private_scoping_preserves_preloaded_authored_package_bindings() {
        let root = tempfile::tempdir().unwrap();
        let selected = source(root.path(), "node_modules/resolved-control/index.ts");
        let package = selected.parent().unwrap().to_path_buf();
        let manifest = package.join("package.json");
        std::fs::write(
            &manifest,
            r#"{"name":"resolved-control","types":"index.ts"}"#,
        )
        .unwrap();
        let mut owner = checker(root.path());
        owner.set_package_routes([crate::PackageRouteBinding {
            importer_path: selected.clone(),
            specifier: "resolved-control".into(),
            occurrence_mode: crate::PackageResolutionMode::Import,
            context: crate::PackageResolutionContext::default(),
            route: Some(crate::PackageRoute {
                source_paths: vec![selected.clone()],
                dependency_paths: vec![],
                source_targets: vec![],
                package_root: package.clone(),
                package_link_root: package,
                manifest_path: manifest.clone(),
                package_name: Some("resolved-control".into()),
                workspace_source: false,
                nested_routes: vec![],
            }),
            invalidation_paths: vec![manifest],
        }]);
        let before = owner.project.package_routes_snapshot();
        assert_eq!(before.len(), 1);
        owner.scope_initial_installed_sources(&[selected]).unwrap();
        assert_eq!(owner.project.package_routes_snapshot(), before);
    }

    #[test]
    fn later_ordinary_owner_expansion_retains_the_documented_persistent_boundary() {
        let root = tempfile::tempdir().unwrap();
        let ordinary = source(root.path(), "src/index.ts");
        let installed = source(root.path(), "node_modules/selected/index.ts");
        let mut owner = checker(root.path());
        owner.scan_paths(&[ordinary]).unwrap();
        let namespace = owner.project.virtual_root().to_path_buf();
        owner.scan_paths(&[installed]).unwrap();
        assert!(owner.owned_storage.is_none());
        assert_eq!(owner.project.virtual_root(), namespace);
    }

    #[cfg(unix)]
    #[test]
    fn canonical_installed_target_selects_private_storage() {
        let root = tempfile::tempdir().unwrap();
        let installed = source(root.path(), "node_modules/selected/index.ts");
        std::fs::create_dir(root.path().join("src")).unwrap();
        let logical = root.path().join("src/installed.ts");
        std::os::unix::fs::symlink(&installed, &logical).unwrap();
        let mut owner = checker(root.path());
        owner.scan_paths(std::slice::from_ref(&logical)).unwrap();
        let storage = owner.owned_storage.as_ref().unwrap().path().to_path_buf();
        drop(owner);
        assert!(!storage.exists());
        assert_eq!(std::fs::read_link(logical).unwrap(), installed);
    }
}
