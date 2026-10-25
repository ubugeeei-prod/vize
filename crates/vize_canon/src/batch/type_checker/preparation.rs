//! A virtual project can validate authored sources before a compiler is required.

use std::path::{Path, PathBuf};

use super::{
    BatchTypeChecker, BatchTypeCheckerOptions, CorsaExecutor, CorsaResult, IncrementalPaths,
    VirtualProject, VirtualTsCheckOptions, scan_prepared_paths,
};

/// Authored-source preparation before binding a real native compiler.
///
/// This owner keeps installed-source storage alive while callers configure and
/// scan the same project that activation passes to the batch checker.
pub struct BatchTypeCheckerPreparation {
    project: VirtualProject,
    scanned: bool,
    incremental_paths: IncrementalPaths,
    owned_storage: Option<tempfile::TempDir>,
}

impl BatchTypeChecker {
    /// Prepare a virtual project without resolving or launching a native compiler.
    pub fn prepare_with_options(
        project_root: &Path,
        options: BatchTypeCheckerOptions,
    ) -> CorsaResult<BatchTypeCheckerPreparation> {
        let mut project = VirtualProject::new(project_root)?;
        project.set_tsconfig_path(options.tsconfig_path);
        project.set_virtual_ts_options(options.virtual_ts_options);
        Ok(BatchTypeCheckerPreparation {
            project,
            scanned: false,
            incremental_paths: IncrementalPaths::new(),
            owned_storage: None,
        })
    }
}

impl BatchTypeCheckerPreparation {
    /// Bind a real compiler after successful preparation, retaining all ownership.
    pub fn activate(self, corsa_path: Option<&Path>) -> CorsaResult<BatchTypeChecker> {
        let executor = CorsaExecutor::with_corsa_path(self.project.project_root(), corsa_path)?;
        Ok(BatchTypeChecker {
            project: self.project,
            executor,
            scanned: self.scanned,
            server_count: None,
            incremental_paths: self.incremental_paths,
            owned_storage: self.owned_storage,
        })
    }

    /// Resolve Vue 3 Options API template bindings (opt-in, standard build).
    pub fn enable_options_api(&mut self) {
        self.project.set_options_api(true);
    }

    /// Enable Vue 2.7 / Nuxt 2 Options API compatibility for generated virtual
    /// files (implies Options API binding resolution plus Nuxt 2 globals).
    pub fn enable_legacy_vue2(&mut self) {
        self.project.set_legacy_vue2(true);
    }

    /// Enable opt-in type-checking of `.jsx`/`.tsx` Vize components (#1497).
    ///
    /// Off by default so mixed Vue/React repositories do not accidentally route
    /// React `.tsx` through the Vue JSX checker. When enabled, `.jsx`/`.tsx`
    /// route through the Vize JSX virtual-TS path instead of the verbatim React
    /// passthrough.
    pub fn enable_jsx_typecheck(&mut self) {
        self.project.set_jsx_typecheck(true);
    }

    /// Configure which virtual TypeScript checks are generated for Vue files.
    pub fn set_virtual_ts_checks(
        &mut self,
        check_props: bool,
        check_template_bindings: bool,
        check_emits: bool,
    ) {
        self.project
            .set_virtual_ts_check_options(VirtualTsCheckOptions {
                check_props,
                check_template_bindings,
                check_emits,
                ..Default::default()
            });
    }

    /// Preserve importer-scoped package identities in the virtual project.
    pub fn set_package_routes(
        &mut self,
        routes: impl IntoIterator<Item = crate::PackageRouteBinding>,
    ) {
        self.project.set_package_routes(routes);
    }

    /// Reuse the route-discovery cache that produced the supplied bindings.
    pub fn set_package_route_resolver(&mut self, resolver: crate::PackageRouteResolver) {
        self.project.set_package_route_resolver(resolver);
    }

    /// Configure template syntax compatibility for Vue template parsing.
    pub fn set_template_syntax(&mut self, template_syntax: vize_atelier_core::TemplateSyntaxMode) {
        self.project.set_template_syntax(template_syntax);
    }

    /// Enable experimental Vue in-tag comments for template parsing.
    pub fn set_experimental_in_tag_comments(&mut self, enabled: bool) {
        self.project.set_experimental_in_tag_comments(enabled);
    }

    /// Enable experimental RFC 823 patterned template type checking.
    pub fn set_experimental_patterned_template(&mut self, enabled: bool) {
        self.project.set_experimental_patterned_template(enabled);
    }

    /// Enable experimental strict checking of provided slot child types.
    pub fn set_experimental_strict_slot_children(&mut self, enabled: bool) {
        self.project.set_experimental_strict_slot_children(enabled);
    }

    /// Set the configured Vue dialect (`vue.version`; default
    /// [`VueVersion::V3`](vize_carton::config::VueVersion::V3)).
    ///
    /// Threaded into virtual-TS generation so canon can emit dialect-aware
    /// instance types.
    pub fn set_dialect(&mut self, dialect: vize_carton::config::VueVersion) {
        self.project.set_dialect(dialect);
    }

    /// Scan an explicit set of project files.
    ///
    /// The underlying virtual project parallelizes registration for multi-file
    /// batches, so callers should prefer passing the whole discovered input set
    /// rather than looping over `register_path` themselves. That keeps SFC parse
    /// and virtual-TS generation CPU-bound instead of serializing every file on
    /// the batch checker.
    pub fn scan_paths(&mut self, paths: &[PathBuf]) -> CorsaResult<()> {
        scan_prepared_paths(
            &mut self.project,
            &mut self.scanned,
            &mut self.incremental_paths,
            &mut self.owned_storage,
            paths,
        )
    }

    /// Register tsconfig `include` sources the file walk did not hand to
    /// `scan_paths`. Explicit subset scans do not call this.
    pub fn ensure_included_sources(&mut self) -> CorsaResult<()> {
        self.project.ensure_included_sources()
    }

    /// Register diagnosed paths that exist as project sources and were never
    /// mirrored. `node_modules` paths stay out; TypeScript loads those itself.
    pub fn adopt_diagnostic_sources(
        &mut self,
        paths: impl IntoIterator<Item = PathBuf>,
    ) -> CorsaResult<()> {
        self.project.adopt_diagnostic_sources(paths)
    }

    /// Diagnose selected authored sources in the prepared project.
    pub fn set_diagnostic_paths<'a>(&mut self, paths: impl IntoIterator<Item = &'a Path>) {
        self.project.set_diagnostic_paths(paths);
    }
}

#[cfg(test)]
mod tests {
    use super::{BatchTypeChecker, BatchTypeCheckerOptions};
    use crate::batch::CorsaError;

    #[test]
    fn preparation_reports_authored_parse_errors_without_resolving_a_compiler() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("Broken.vue");
        std::fs::write(&source, "<template><div>").unwrap();
        let mut prepared =
            BatchTypeChecker::prepare_with_options(root.path(), BatchTypeCheckerOptions::default())
                .unwrap();
        assert!(matches!(
            prepared.scan_paths(&[source]),
            Err(CorsaError::SfcParse(_))
        ));
        assert!(!prepared.scanned);
        let missing = root.path().join("missing-native-runtime");
        assert!(matches!(
            BatchTypeChecker::with_options_and_corsa_path(
                root.path(),
                BatchTypeCheckerOptions::default(),
                Some(&missing),
            ),
            Err(CorsaError::CorsaNotFound(_))
        ));
    }

    #[test]
    fn activation_reuses_the_prepared_scan_and_cleans_private_storage_on_failure() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("node_modules/authored/Selected.vue");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, "<template><p /></template>\n").unwrap();
        let mut prepared =
            BatchTypeChecker::prepare_with_options(root.path(), BatchTypeCheckerOptions::default())
                .unwrap();
        prepared.scan_paths(std::slice::from_ref(&source)).unwrap();
        assert!(prepared.scanned);
        let storage = prepared
            .owned_storage
            .as_ref()
            .unwrap()
            .path()
            .to_path_buf();
        assert!(storage.is_dir());
        // Activation must not read or parse the source again.
        std::fs::write(&source, "<template><p>").unwrap();
        let missing = root.path().join("missing-native-runtime");
        assert!(matches!(
            prepared.activate(Some(&missing)),
            Err(CorsaError::CorsaNotFound(_))
        ));
        assert!(!storage.exists());
    }

    #[test]
    fn failed_installed_source_preparation_releases_its_private_namespace() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("node_modules/authored/Broken.vue");
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::write(&source, "<template><p>").unwrap();
        let mut prepared =
            BatchTypeChecker::prepare_with_options(root.path(), BatchTypeCheckerOptions::default())
                .unwrap();
        assert!(matches!(
            prepared.scan_paths(&[source]),
            Err(CorsaError::SfcParse(_))
        ));
        let storage = prepared
            .owned_storage
            .as_ref()
            .unwrap()
            .path()
            .to_path_buf();
        assert!(storage.is_dir());
        drop(prepared);
        assert!(!storage.exists());
    }
}
