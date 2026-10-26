//! Error types for batch type checking.

use std::path::{Path, PathBuf};
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;

/// Error type for Corsa-backed batch operations.
#[derive(Debug, thiserror::Error)]
pub enum CorsaError {
    /// Corsa executable not found.
    #[error("{0}")]
    CorsaNotFound(#[from] CorsaNotFoundError),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// SFC parse error.
    #[error("SFC parse error: {0}")]
    SfcParse(String),

    /// Path error.
    #[error("Path error: {path}")]
    PathError { path: PathBuf },

    /// Corsa returned an error.
    #[error("corsa error (exit code {exit_code}): {message}")]
    CorsaExecution { exit_code: i32, message: String },

    /// JSON parse error.
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// Invalid authored project configuration.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Project not initialized.
    #[error("Virtual project not initialized. Call scan_project() first.")]
    NotInitialized,

    /// Strip prefix error.
    #[error("Failed to strip prefix from path: {0}")]
    StripPrefix(#[from] std::path::StripPrefixError),

    /// Walkdir error.
    #[error("Directory walk error: {0}")]
    WalkDir(#[from] walkdir::Error),
}

/// Result type for Corsa-backed batch operations.
pub type CorsaResult<T> = Result<T, CorsaError>;

/// Package manager type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Pnpm,
    Npm,
    Yarn,
    Bun,
}

/// Error when a Corsa executable cannot be resolved.
#[derive(Debug)]
pub struct CorsaNotFoundError {
    detected_pm: Option<PackageManager>,
    explicit_path: Option<PathBuf>,
    invalid_runtime: Option<PathBuf>,
}

impl CorsaNotFoundError {
    /// Create a new CorsaNotFoundError.
    pub fn new(project_root: &Path) -> Self {
        let detected_pm = detect_package_manager(project_root);
        Self {
            detected_pm,
            explicit_path: None,
            invalid_runtime: None,
        }
    }

    /// Create a new CorsaNotFoundError for an explicit but missing path.
    pub fn new_explicit(project_root: &Path, path: &Path) -> Self {
        let detected_pm = detect_package_manager(project_root);
        Self {
            detected_pm,
            explicit_path: Some(path.to_path_buf()),
            invalid_runtime: None,
        }
    }

    /// Create an error for an installed but incomplete or mismatched runtime.
    pub fn new_invalid(project_root: &Path, directory: &Path) -> Self {
        Self {
            detected_pm: detect_package_manager(project_root),
            explicit_path: None,
            invalid_runtime: Some(directory.to_path_buf()),
        }
    }

    /// Get the detected package manager.
    pub fn detected_package_manager(&self) -> Option<PackageManager> {
        self.detected_pm
    }

    /// Generate CLI error message with installation instructions.
    pub fn display_message(&self) -> String {
        let mut msg = String::default();

        if let Some(directory) = &self.invalid_runtime {
            msg.push_str("error: native TypeScript runtime is invalid\n\n");
            append!(
                msg,
                "The runtime declared under {} is incomplete or mismatched (platform package version, package.json, or lib/tsc).\n\n",
                directory.display()
            );
            msg.push_str("Reinstall that package, or set typeChecker.corsaPath, CORSA_PATH, or TSGO_PATH to an existing native executable.\n");
            return msg;
        }
        msg.push_str("error: native TypeScript runtime not found\n\n");
        if let Some(path) = &self.explicit_path {
            append!(
                msg,
                "Configured Corsa executable does not exist: {}\n\n",
                path.display()
            );
            msg.push_str("Correct the configured path, or set typeChecker.corsaPath, CORSA_PATH, or TSGO_PATH to an existing native executable.\n");
            return msg;
        }
        msg.push_str("vize check requires a user-installed native TypeScript/tsgo executable.\n");
        msg.push_str("Keep your existing 'typescript' package. Install a native compiler under a separate alias, or configure typeChecker.corsaPath, CORSA_PATH, or TSGO_PATH.\n\n");
        msg.push_str("The example below selects native TypeScript 7.1.0-beta; choose the native version you want to run.\n\n");

        if let Some(pm) = self.detected_pm {
            msg.push_str("Example install command:\n\n");
            append!(msg, "  {}\n", self.install_command(pm));
        } else {
            msg.push_str("Example install commands (choose your package manager):\n\n");
            append!(
                msg,
                "  {}  # npm\n",
                self.install_command(PackageManager::Npm)
            );
            append!(
                msg,
                "  {}  # pnpm\n",
                self.install_command(PackageManager::Pnpm)
            );
            append!(
                msg,
                "  {}  # yarn\n",
                self.install_command(PackageManager::Yarn)
            );
            append!(
                msg,
                "  {}  # bun\n",
                self.install_command(PackageManager::Bun)
            );
        }

        msg
    }

    fn install_command(&self, pm: PackageManager) -> String {
        match pm {
            PackageManager::Npm => {
                cstr!("npm install -D typescript-native@npm:typescript@7.1.0-beta")
            }
            PackageManager::Pnpm => {
                cstr!("pnpm add -D typescript-native@npm:typescript@7.1.0-beta")
            }
            PackageManager::Yarn => {
                cstr!("yarn add -D typescript-native@npm:typescript@7.1.0-beta")
            }
            PackageManager::Bun => cstr!("bun add -D typescript-native@npm:typescript@7.1.0-beta"),
        }
    }
}

impl std::fmt::Display for CorsaNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_message())
    }
}

impl std::error::Error for CorsaNotFoundError {}

/// Detect the project's package manager.
///
/// A workspace package has no lockfile of its own, so the search continues in
/// the parent directories and the nearest answer wins. It stops at the
/// repository root: a lockfile above the directory that holds `.git` belongs
/// to something else. `npm/cli/src/init/detect.ts` mirrors this search.
pub fn detect_package_manager(project_root: &Path) -> Option<PackageManager> {
    let absolute = std::path::absolute(project_root);
    let start = absolute.as_deref().unwrap_or(project_root);
    for directory in start.ancestors() {
        if let Some(detected) = detect_package_manager_in(directory) {
            return Some(detected);
        }
        if directory.join(".git").exists() {
            return None;
        }
    }
    None
}

fn detect_package_manager_in(project_root: &Path) -> Option<PackageManager> {
    // 1. Detect from lockfile (priority order)
    if project_root.join("pnpm-lock.yaml").exists() {
        return Some(PackageManager::Pnpm);
    }
    if project_root.join("bun.lockb").exists() || project_root.join("bun.lock").exists() {
        return Some(PackageManager::Bun);
    }
    if project_root.join("yarn.lock").exists() {
        return Some(PackageManager::Yarn);
    }
    if project_root.join("package-lock.json").exists() {
        return Some(PackageManager::Npm);
    }

    // 2. Detect from package.json packageManager field
    let pkg_json = project_root.join("package.json");
    if let Ok(content) = std::fs::read_to_string(&pkg_json)
        && let Ok(json) = serde_json::from_str::<serde_json::Value>(&content)
        && let Some(pm) = json.get("packageManager").and_then(|v| v.as_str())
    {
        if pm.starts_with("pnpm") {
            return Some(PackageManager::Pnpm);
        }
        if pm.starts_with("yarn") {
            return Some(PackageManager::Yarn);
        }
        if pm.starts_with("bun") {
            return Some(PackageManager::Bun);
        }
        if pm.starts_with("npm") {
            return Some(PackageManager::Npm);
        }
    }

    None
}

#[expect(
    clippy::disallowed_macros,
    reason = "insta snapshots expand to format!"
)]
#[cfg(test)]
mod tests {
    use super::{CorsaNotFoundError, PackageManager, detect_package_manager};

    #[test]
    fn workspace_package_uses_the_workspace_package_manager() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        std::fs::create_dir(workspace.path().join(".git")).expect("repository root");
        std::fs::write(workspace.path().join("pnpm-lock.yaml"), "").expect("lockfile");
        let package = workspace.path().join("packages").join("app");
        std::fs::create_dir_all(&package).expect("workspace package");
        std::fs::write(package.join("package.json"), "{}").expect("package manifest");

        assert_eq!(detect_package_manager(&package), Some(PackageManager::Pnpm));

        std::fs::write(package.join("package-lock.json"), "{}").expect("nearer lockfile");
        assert_eq!(detect_package_manager(&package), Some(PackageManager::Npm));
    }

    #[test]
    fn lockfile_above_the_repository_root_is_ignored() {
        let outside = tempfile::tempdir().expect("temp directory");
        std::fs::write(outside.path().join("yarn.lock"), "").expect("unrelated lockfile");
        let repository = outside.path().join("repository");
        let package = repository.join("packages").join("app");
        std::fs::create_dir_all(&package).expect("workspace package");
        std::fs::create_dir(repository.join(".git")).expect("repository root");

        assert_eq!(detect_package_manager(&repository), None);
        assert_eq!(detect_package_manager(&package), None);
    }

    #[test]
    fn test_corsa_not_found_error_message() {
        let error = CorsaNotFoundError {
            detected_pm: Some(PackageManager::Pnpm),
            explicit_path: None,
            invalid_runtime: None,
        };

        let msg = error.display_message();
        insta::assert_snapshot!(msg.as_str());
    }

    #[test]
    fn test_corsa_not_found_no_pm() {
        let error = CorsaNotFoundError {
            detected_pm: None,
            explicit_path: None,
            invalid_runtime: None,
        };

        let msg = error.display_message();
        insta::assert_snapshot!(msg.as_str());
    }
    #[test]
    fn explicit_missing_runtime_reports_the_full_path_without_install_advice() {
        let error = CorsaNotFoundError {
            detected_pm: Some(PackageManager::Npm),
            explicit_path: Some(std::path::PathBuf::from("/project/bin/missing-tsgo")),
            invalid_runtime: None,
        };
        assert_eq!(
            error.display_message().as_str(),
            "error: native TypeScript runtime not found\n\nConfigured Corsa executable does not exist: /project/bin/missing-tsgo\n\nCorrect the configured path, or set typeChecker.corsaPath, CORSA_PATH, or TSGO_PATH to an existing native executable.\n"
        );
    }

    #[test]
    fn invalid_declared_runtime_names_its_node_modules_without_install_advice() {
        let error = CorsaNotFoundError {
            detected_pm: Some(PackageManager::Npm),
            explicit_path: None,
            invalid_runtime: Some(std::path::PathBuf::from("/project/node_modules")),
        };
        assert_eq!(
            error.display_message().as_str(),
            "error: native TypeScript runtime is invalid\n\nThe runtime declared under /project/node_modules is incomplete or mismatched (platform package version, package.json, or lib/tsc).\n\nReinstall that package, or set typeChecker.corsaPath, CORSA_PATH, or TSGO_PATH to an existing native executable.\n"
        );
    }

    #[test]
    fn every_package_manager_keeps_the_existing_typescript_dependency() {
        let error = CorsaNotFoundError {
            detected_pm: None,
            explicit_path: None,
            invalid_runtime: None,
        };
        for (manager, expected) in [
            (
                PackageManager::Npm,
                "npm install -D typescript-native@npm:typescript@7.1.0-beta",
            ),
            (
                PackageManager::Pnpm,
                "pnpm add -D typescript-native@npm:typescript@7.1.0-beta",
            ),
            (
                PackageManager::Yarn,
                "yarn add -D typescript-native@npm:typescript@7.1.0-beta",
            ),
            (
                PackageManager::Bun,
                "bun add -D typescript-native@npm:typescript@7.1.0-beta",
            ),
        ] {
            assert_eq!(error.install_command(manager).as_str(), expected);
        }
    }
}
