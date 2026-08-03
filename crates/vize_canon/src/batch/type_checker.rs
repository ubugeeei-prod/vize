//! TypeChecker trait and BatchTypeChecker implementation.

use std::path::{Path, PathBuf};

use super::Diagnostic;
use super::error::{CorsaError, CorsaResult};
use super::executor::CorsaExecutor;
use super::virtual_project::VirtualProject;
use crate::virtual_ts::{VirtualTsCheckOptions, VirtualTsOptions};
use vize_carton::String;

mod metrics;
mod paths;
mod result;
pub use metrics::IncrementalCheckMetrics;
use paths::{IncrementalPaths, collect_project_paths};

/// Result of type checking.
#[derive(Debug, Default)]
pub struct TypeCheckResult {
    /// Diagnostics from type checking.
    pub diagnostics: Vec<Diagnostic>,
    /// Exit code from the Corsa process.
    pub exit_code: i32,
    /// Whether type checking succeeded.
    pub success: bool,
}

/// Options for a project-backed batch type checker.
#[derive(Debug, Clone, Default)]
pub struct BatchTypeCheckerOptions {
    /// Explicit tsconfig path to extend for the materialized project.
    pub tsconfig_path: Option<PathBuf>,
    /// Shared Vue virtual TS options.
    pub virtual_ts_options: VirtualTsOptions,
}

/// Options for declaration emit.
#[derive(Debug, Clone)]
pub struct DeclarationEmitOptions {
    /// Output directory where emitted `.d.ts` files should be written.
    pub out_dir: PathBuf,
    /// Whether declaration maps should be emitted as well.
    pub declaration_map: bool,
}

impl DeclarationEmitOptions {
    /// Create declaration emit options for the given output directory.
    pub fn new(out_dir: PathBuf) -> Self {
        Self {
            out_dir,
            declaration_map: false,
        }
    }

    /// Enable or disable declaration map emit.
    pub fn with_declaration_map(mut self, declaration_map: bool) -> Self {
        self.declaration_map = declaration_map;
        self
    }
}

/// A single emitted declaration file.
#[derive(Debug, Clone)]
pub struct DeclarationOutput {
    /// Absolute emitted file path.
    pub path: PathBuf,
    /// Emitted file content.
    pub content: String,
}

/// Result of declaration emit.
#[derive(Debug, Clone, Default)]
pub struct DeclarationEmitResult {
    /// Emitted declaration files.
    pub files: Vec<DeclarationOutput>,
}

/// Trait for type checking.
pub trait TypeChecker: Send + Sync {
    /// Check the entire project.
    fn check_project(&self) -> CorsaResult<TypeCheckResult>;

    /// Check a single file.
    fn check_file(&self, path: &Path, content: &str) -> CorsaResult<Vec<Diagnostic>>;

    /// Re-check a scanned project after source files changed.
    ///
    /// The result must observe the latest on-disk contents and include
    /// diagnostics from dependents, not only the changed files themselves.
    ///
    /// This is a low-level, Corsa-facing embedding API; it currently has no
    /// in-tree production consumer. Vize rebuilds the scanned virtual sources
    /// and sends their materialized delta to the persistent Corsa project
    /// session. Corsa, rather than a duplicate Vize dependency graph, owns
    /// dependency-aware invalidation. Use
    /// [`BatchTypeChecker::incremental_metrics`] to measure session reuse,
    /// refresh scope, diagnostic requests, and CLI degradation.
    fn check_incremental(&self, changed: &[PathBuf]) -> CorsaResult<TypeCheckResult>;
}

/// Batch type checker using the Corsa CLI.
pub struct BatchTypeChecker {
    /// Virtual project.
    project: VirtualProject,
    /// Corsa executor.
    executor: CorsaExecutor,
    /// Whether the project has been scanned.
    scanned: bool,
    /// Number of parallel Corsa CLI processes; `None` auto-tunes.
    server_count: Option<usize>,
    /// Source membership carried across incremental checks.
    incremental_paths: IncrementalPaths,
}

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
        })
    }

    /// Set the number of parallel Corsa CLI processes the project check is
    /// partitioned across. `None` (the default) auto-tunes from the machine
    /// width and the number of registered Vue files.
    pub fn set_server_count(&mut self, servers: Option<usize>) {
        self.server_count = servers;
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
            });
    }

    /// Configure template syntax compatibility for Vue template parsing.
    pub fn set_template_syntax(&mut self, template_syntax: vize_atelier_core::TemplateSyntaxMode) {
        self.project.set_template_syntax(template_syntax);
    }

    /// Enable experimental Vue in-tag comments for template parsing.
    pub fn set_experimental_in_tag_comments(&mut self, enabled: bool) {
        self.project.set_experimental_in_tag_comments(enabled);
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
        self.project.register_paths(paths)?;
        self.scanned = true;
        self.incremental_paths.after_explicit_scan(paths);
        Ok(())
    }

    /// Scan the project for source files.
    pub fn scan_project(&mut self) -> CorsaResult<()> {
        let paths = collect_project_paths(self.project.project_root())?;
        self.project.register_paths(&paths)?;
        self.scanned = true;
        self.incremental_paths.after_project_scan(&self.project);
        Ok(())
    }

    /// Get the number of registered files.
    pub fn file_count(&self) -> usize {
        self.project.file_count()
    }

    /// Access the materialized virtual files in deterministic order.
    pub fn virtual_files(&self) -> Vec<&super::virtual_project::VirtualFile> {
        self.project.virtual_files_sorted()
    }

    /// Access the shared helpers preamble when this checker generated one.
    pub fn shared_helpers_preamble(&self) -> Option<&'static str> {
        self.project
            .uses_shared_helpers()
            .then_some(crate::virtual_ts::SHARED_PREAMBLE_DTS)
    }

    /// Emit declaration files for the scanned project.
    pub fn emit_declarations(
        &self,
        options: &DeclarationEmitOptions,
    ) -> CorsaResult<DeclarationEmitResult> {
        if !self.scanned {
            return Err(CorsaError::NotInitialized);
        }

        self.executor.emit_declarations(&self.project, options)
    }
}

impl TypeChecker for BatchTypeChecker {
    fn check_project(&self) -> CorsaResult<TypeCheckResult> {
        if !self.scanned {
            return Err(CorsaError::NotInitialized);
        }

        self.check_registered_project(&self.project)
    }

    fn check_file(&self, path: &Path, content: &str) -> CorsaResult<Vec<Diagnostic>> {
        // Create a temporary project with just this file
        let project_root = path.parent().unwrap_or(Path::new("."));
        let mut temp_project = VirtualProject::new(project_root)?;
        temp_project.register_path_with_content(path, content)?;

        let mut result = self.executor.check(&temp_project)?;
        result
            .diagnostics
            .extend(temp_project.diagnostics().iter().cloned());
        Ok(result.diagnostics)
    }

    fn check_incremental(&self, changed: &[PathBuf]) -> CorsaResult<TypeCheckResult> {
        if !self.scanned {
            return Err(CorsaError::NotInitialized);
        }
        if changed.is_empty() {
            return self.check_project();
        }

        // Rebuilding source-derived virtual files preserves the observable
        // correctness contract; the executor reuses Corsa's dependency graph
        // after diffing the complete materialized snapshot.
        let paths = self.incremental_paths.refresh(&self.project, changed)?;

        let mut refreshed = self.project.empty_with_same_options()?;
        refreshed.register_paths(&paths)?;
        self.check_registered_project_incremental(&refreshed)
    }
}

impl BatchTypeChecker {
    fn check_registered_project(&self, project: &VirtualProject) -> CorsaResult<TypeCheckResult> {
        let result = self
            .executor
            .check_with_servers(project, self.server_count)?;
        Self::finish_registered_result(result, project)
    }

    fn check_registered_project_incremental(
        &self,
        project: &VirtualProject,
    ) -> CorsaResult<TypeCheckResult> {
        let result = self
            .executor
            .check_incremental_session(project, self.server_count)?;
        Self::finish_registered_result(result, project)
    }

    fn finish_registered_result(
        mut result: TypeCheckResult,
        project: &VirtualProject,
    ) -> CorsaResult<TypeCheckResult> {
        result
            .diagnostics
            .extend(project.diagnostics().iter().cloned());
        if result.has_errors() {
            result.success = false;
            result.exit_code = result.exit_code.max(1);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests;
