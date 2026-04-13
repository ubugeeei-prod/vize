//! TypeChecker trait and BatchTypeChecker implementation.

use std::path::{Path, PathBuf};

use super::error::{CorsaError, CorsaResult};
use super::executor::CorsaExecutor;
use super::virtual_project::VirtualProject;
use super::Diagnostic;
use crate::virtual_ts::VirtualTsOptions;
use vize_carton::String;

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

impl TypeCheckResult {
    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == 1)
    }

    /// Get the number of errors.
    pub fn error_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity == 1).count()
    }

    /// Get the number of warnings.
    pub fn warning_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity == 2).count()
    }
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

    /// Check incrementally (only changed files).
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
        let project = VirtualProject::new(project_root)?;
        let mut project = project;
        project.set_tsconfig_path(options.tsconfig_path);
        project.set_virtual_ts_options(options.virtual_ts_options);
        let executor = CorsaExecutor::new(project_root)?;

        Ok(Self {
            project,
            executor,
            scanned: false,
        })
    }

    /// Scan an explicit set of project files.
    pub fn scan_paths(&mut self, paths: &[PathBuf]) -> CorsaResult<()> {
        for path in paths {
            if !path.is_file() {
                continue;
            }
            self.project.register_path(path)?;
        }
        self.scanned = true;
        Ok(())
    }

    /// Scan the project for source files.
    pub fn scan_project(&mut self) -> CorsaResult<()> {
        let project_root = self.project.project_root().to_path_buf();

        for entry in walkdir::WalkDir::new(&project_root)
            .into_iter()
            .filter_entry(|e| {
                // Don't filter the root directory itself
                if e.path() == project_root {
                    return true;
                }
                // Skip node_modules and hidden directories
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && name != "node_modules"
            })
        {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if !is_supported_input(path) {
                continue;
            }

            self.project.register_path(path)?;
        }

        self.scanned = true;
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

        self.executor.check(&self.project)
    }

    fn check_file(&self, path: &Path, content: &str) -> CorsaResult<Vec<Diagnostic>> {
        // Create a temporary project with just this file
        let project_root = path.parent().unwrap_or(Path::new("."));
        let mut temp_project = VirtualProject::new(project_root)?;
        temp_project.register_path_with_content(path, content)?;

        let result = self.executor.check(&temp_project)?;
        Ok(result.diagnostics)
    }

    fn check_incremental(&self, changed: &[PathBuf]) -> CorsaResult<TypeCheckResult> {
        // For now, just do a full check
        // TODO: Implement proper incremental checking
        let _ = changed;
        self.check_project()
    }
}

fn is_supported_input(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "vue" | "ts" | "tsx" | "mts" | "cts"))
        || path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".d.ts"))
}

#[cfg(test)]
mod tests;
