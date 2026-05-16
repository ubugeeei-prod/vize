//! Error types for batch type checking.

use std::path::{Path, PathBuf};
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::String;

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
}

impl CorsaNotFoundError {
    /// Create a new CorsaNotFoundError.
    pub fn new(project_root: &Path) -> Self {
        let detected_pm = detect_package_manager(project_root);
        Self { detected_pm }
    }

    /// Get the detected package manager.
    pub fn detected_package_manager(&self) -> Option<PackageManager> {
        self.detected_pm
    }

    /// Generate CLI error message with installation instructions.
    pub fn display_message(&self) -> String {
        let mut msg = String::default();

        msg.push_str("error: corsa not found\n\n");
        msg.push_str("vize check requires '@typescript/native-preview' to be installed.\n\n");

        if let Some(pm) = self.detected_pm {
            msg.push_str("To install, run:\n\n");
            append!(msg, "  {}\n", self.install_command(pm));
        } else {
            msg.push_str("To install, run one of the following:\n\n");
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
            PackageManager::Npm => cstr!("npm install -D @typescript/native-preview"),
            PackageManager::Pnpm => cstr!("pnpm add -D @typescript/native-preview"),
            PackageManager::Yarn => cstr!("yarn add -D @typescript/native-preview"),
            PackageManager::Bun => cstr!("bun add -D @typescript/native-preview"),
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
pub fn detect_package_manager(project_root: &Path) -> Option<PackageManager> {
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
    if let Ok(content) = std::fs::read_to_string(&pkg_json) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(pm) = json.get("packageManager").and_then(|v| v.as_str()) {
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
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{CorsaNotFoundError, PackageManager};

    #[test]
    fn test_corsa_not_found_error_message() {
        let error = CorsaNotFoundError {
            detected_pm: Some(PackageManager::Pnpm),
        };

        let msg = error.display_message();
        insta::assert_snapshot!(msg.as_str());
    }

    #[test]
    fn test_corsa_not_found_no_pm() {
        let error = CorsaNotFoundError { detected_pm: None };

        let msg = error.display_message();
        insta::assert_snapshot!(msg.as_str());
    }
}
