//! Unified Corsa/tsgo executable discovery.
//!
//! Historically three independent resolvers existed (canon's LSP client, canon's
//! batch executor, and patina's type-aware linter session) and they could pick
//! different binaries for the same project. This module is the single source of
//! truth they all share.
//!
//! Resolution order:
//!
//! 1. Explicit configuration ([`CorsaResolveRequest::explicit_path`]).
//! 2. Environment variables, in precedence order: `CORSA_PATH`,
//!    `CORSA_EXECUTABLE`, `TSGO_PATH`, `TSGO_EXECUTABLE`. The first one set
//!    wins; its value must point at an existing file.
//! 3. Project discovery: an ancestor walk from the project root (and the
//!    current directory) probing project caches, dev-checkout paths, stable
//!    `typescript@7` platform packages, the bundled `node_modules/vize`
//!    runtime, legacy `@typescript/native-preview` layouts, pnpm store
//!    fallbacks, and finally local `node_modules/.bin` wrappers.
//! 4. Common global install locations under `$HOME` plus `npm root -g`.
//! 5. A `PATH` lookup for `corsa` then `tsgo`.
//!
//! Developer-checkout paths (`ref/typescript-go/...`, `../corsa-bind`, and the
//! compile-time workspace root) are probed only in debug builds or when
//! `VIZE_CORSA_DEV_PATHS` is set to a non-empty value other than `0`, so
//! production binaries never scan repo-internal layouts.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod normalize_tests;

/// Environment variables honored by the resolver, in precedence order.
pub const CORSA_ENV_VARS: [&str; 4] = [
    "CORSA_PATH",
    "CORSA_EXECUTABLE",
    "TSGO_PATH",
    "TSGO_EXECUTABLE",
];

/// Executable names probed by every discovery source, in precedence order.
pub const CORSA_EXECUTABLE_NAMES: [&str; 2] = ["corsa", "tsgo"];

/// Environment variable gating developer-checkout discovery paths.
pub const CORSA_DEV_PATHS_ENV: &str = "VIZE_CORSA_DEV_PATHS";

/// Input for [`resolve_corsa_executable`].
#[derive(Debug, Clone, Copy, Default)]
pub struct CorsaResolveRequest<'a> {
    /// Explicitly configured executable path (CLI flag or config file).
    /// Takes precedence over everything else and must exist.
    pub explicit_path: Option<&'a Path>,
    /// Project root used to anchor relative explicit/env paths and the
    /// discovery ancestor walk.
    pub project_root: Option<&'a Path>,
}

/// Failure modes of [`resolve_corsa_executable`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorsaResolveError {
    /// An explicitly requested path (config or environment variable) does not
    /// exist on disk.
    ExplicitNotFound {
        /// Where the path came from: `"configuration"` or an env var name.
        source: &'static str,
        /// The (relative-resolved) path that was probed.
        path: PathBuf,
    },
    /// No executable was found by any discovery source.
    NotFound,
}

impl std::fmt::Display for CorsaResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExplicitNotFound { source, path } => write!(
                f,
                "Configured Corsa executable does not exist: {} (from {source})",
                path.display()
            ),
            Self::NotFound => write!(f, "No Corsa executable found"),
        }
    }
}

impl std::error::Error for CorsaResolveError {}

/// Resolve the Corsa executable for a project.
///
/// See the module docs for the full resolution order.
pub fn resolve_corsa_executable(
    request: CorsaResolveRequest<'_>,
) -> Result<PathBuf, CorsaResolveError> {
    resolve_with_env(request, |name| std::env::var_os(name))
}

/// Discover a Corsa executable by walking `start` and its ancestors, without
/// consulting env vars, `$HOME`, `npm root -g`, or `PATH`.
///
/// This is the discovery subset used to normalize wrapper paths; it is also
/// useful for tests that need a project-scoped lookup.
pub fn discover_corsa_in_ancestors(start: &Path) -> Option<PathBuf> {
    discover_in_walk(&[start.to_path_buf()], dev_paths_enabled())
}

/// Re-resolve a `node_modules/.bin` wrapper path to the native binary it
/// shadows, when one can be discovered from the wrapper's project root.
/// Non-wrapper paths are returned unchanged.
pub fn normalize_corsa_path(path: &Path) -> PathBuf {
    normalize_corsa_path_with_discovery(path, |project_root| {
        discover_in_walk(&[project_root.to_path_buf()], dev_paths_enabled())
    })
}

fn normalize_corsa_path_with_discovery(
    path: &Path,
    discover: impl FnOnce(&Path) -> Option<PathBuf>,
) -> PathBuf {
    let Some(project_root) = wrapper_project_root(path) else {
        return path.to_path_buf();
    };
    match discover(project_root) {
        Some(resolved) if resolved != path => resolved,
        _ => path.to_path_buf(),
    }
}

/// The `@typescript/*-*` platform suffix for the current target.
pub fn platform_suffix() -> &'static str {
    if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "darwin-arm64"
        } else {
            "darwin-x64"
        }
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "aarch64") {
            "linux-arm64"
        } else {
            "linux-x64"
        }
    } else if cfg!(target_os = "windows") {
        if cfg!(target_arch = "aarch64") {
            "win32-arm64"
        } else {
            "win32-x64"
        }
    } else {
        ""
    }
}

fn resolve_with_env(
    request: CorsaResolveRequest<'_>,
    env: impl Fn(&str) -> Option<OsString>,
) -> Result<PathBuf, CorsaResolveError> {
    if let Some(path) = request.explicit_path {
        return resolve_required_path(path, "configuration", request.project_root);
    }

    for env_var in CORSA_ENV_VARS {
        let Some(value) = env(env_var) else {
            continue;
        };
        if value.is_empty() {
            continue;
        }
        return resolve_required_path(Path::new(&value), env_var, request.project_root);
    }

    discover_runtime(request.project_root).ok_or(CorsaResolveError::NotFound)
}

/// Resolve an explicitly requested path: anchor relative paths, require
/// existence, normalize wrappers, and canonicalize.
fn resolve_required_path(
    path: &Path,
    source: &'static str,
    project_root: Option<&Path>,
) -> Result<PathBuf, CorsaResolveError> {
    let resolved = resolve_relative_path(path, project_root);
    if !resolved.exists() {
        return Err(CorsaResolveError::ExplicitNotFound {
            source,
            path: resolved,
        });
    }
    let normalized = normalize_corsa_path(&resolved);
    Ok(normalized.canonicalize().unwrap_or(normalized))
}

fn resolve_relative_path(path: &Path, project_root: Option<&Path>) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }

    let project_candidate = project_root.map(|root| root.join(path));
    if let Some(candidate) = &project_candidate
        && candidate.exists()
    {
        return candidate.clone();
    }

    if let Ok(cwd) = std::env::current_dir() {
        let cwd_candidate = cwd.join(path);
        if cwd_candidate.exists() {
            return cwd_candidate;
        }
    }

    project_candidate.unwrap_or_else(|| path.to_path_buf())
}

fn discover_runtime(project_root: Option<&Path>) -> Option<PathBuf> {
    let dev_paths = dev_paths_enabled();
    let mut walk_starts = Vec::new();
    if let Some(root) = project_root {
        push_unique(&mut walk_starts, root.to_path_buf());
    }
    if let Ok(cwd) = std::env::current_dir() {
        push_unique(&mut walk_starts, cwd);
    }
    if dev_paths && let Some(workspace_root) = compile_time_workspace_root() {
        push_unique(&mut walk_starts, workspace_root);
    }

    if let Some(found) = discover_in_walk(&walk_starts, dev_paths) {
        return Some(found);
    }
    if let Some(found) = find_in_home_locations() {
        return Some(found);
    }
    if let Some(found) = find_in_npm_global_root() {
        return Some(found);
    }
    find_in_path_lookup().map(|path| normalize_corsa_path(&path))
}

/// Whether developer-checkout paths may be probed.
fn dev_paths_enabled() -> bool {
    match std::env::var_os(CORSA_DEV_PATHS_ENV) {
        Some(value) => !value.is_empty() && value != "0",
        None => cfg!(debug_assertions),
    }
}

/// The vize workspace root captured at compile time. Only meaningful for
/// binaries running on a machine with the vize checkout (i.e. development).
fn compile_time_workspace_root() -> Option<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
}

enum Candidate {
    Native(PathBuf),
    Wrapper(PathBuf),
}

/// Walk every start directory and its ancestors, preferring a native binary
/// found anywhere over a `node_modules/.bin` wrapper found anywhere.
fn discover_in_walk(walk_starts: &[PathBuf], dev_paths: bool) -> Option<PathBuf> {
    let mut wrapper_fallback: Option<PathBuf> = None;
    let mut visited: Vec<&Path> = Vec::new();

    for start in walk_starts {
        for dir in start.ancestors() {
            if visited.contains(&dir) {
                continue;
            }
            visited.push(dir);
            match probe_directory(dir, dev_paths) {
                Some(Candidate::Native(path)) => return Some(path),
                Some(Candidate::Wrapper(path)) => {
                    wrapper_fallback.get_or_insert(path);
                }
                None => {}
            }
        }
    }

    wrapper_fallback
}

fn probe_directory(dir: &Path, dev_paths: bool) -> Option<Candidate> {
    if let Some(path) = find_project_cache(dir, dev_paths) {
        return Some(Candidate::Native(path));
    }
    if dev_paths
        && let Some(parent) = dir.parent()
        && let Some(path) = find_project_cache(&parent.join("corsa-bind"), dev_paths)
    {
        return Some(Candidate::Native(path));
    }
    if let Some(path) = find_node_modules_native(dir) {
        return Some(Candidate::Native(path));
    }
    find_node_modules_wrapper(dir).map(Candidate::Wrapper)
}

fn find_project_cache(dir: &Path, dev_paths: bool) -> Option<PathBuf> {
    let mut cache_dirs = vec![dir.join(".cache")];
    if dev_paths {
        let typescript_go = dir.join("ref").join("typescript-go");
        cache_dirs.push(typescript_go.join(".cache"));
        cache_dirs.push(typescript_go.join("built").join("local"));
    }

    for cache_dir in cache_dirs {
        for executable in CORSA_EXECUTABLE_NAMES {
            if let Some(found) = existing_executable(&cache_dir, executable) {
                return Some(found);
            }
        }
    }

    None
}

/// Probe `dir/{name}` and `dir/{name}.exe`. npm's Windows platform packages
/// ship `lib/tsgo.exe`; every other platform ships an extensionless binary.
fn existing_executable(dir: &Path, executable: &str) -> Option<PathBuf> {
    let plain = dir.join(executable);
    if plain.exists() {
        return Some(plain);
    }
    let windows = dir.join(&*crate::cstr!("{executable}.exe"));
    if windows.exists() {
        return Some(windows);
    }
    None
}

fn find_node_modules_native(dir: &Path) -> Option<PathBuf> {
    let node_modules = dir.join("node_modules");
    let suffix = platform_suffix();

    if !suffix.is_empty() {
        if let Some(path) = find_stable_typescript_native(&node_modules, suffix) {
            return Some(path);
        }
        if let Some(path) = find_vize_owned_stable_typescript_native(&node_modules, suffix) {
            return Some(path);
        }

        // Node-style resolution: let the meta package's manifest tell us which
        // platform package to look for, then resolve it like `require` would.
        if let Some(path) = resolve_native_preview_platform_package(&node_modules, suffix) {
            return Some(path);
        }

        let lib_dir = native_preview_platform_package_root(&node_modules, suffix).join("lib");
        for executable in CORSA_EXECUTABLE_NAMES {
            if let Some(found) = existing_executable(&lib_dir, executable) {
                return Some(found);
            }
        }
    }

    let meta_lib_dir = node_modules
        .join("@typescript")
        .join("native-preview")
        .join("lib");
    for executable in CORSA_EXECUTABLE_NAMES {
        if let Some(found) = existing_executable(&meta_lib_dir, executable) {
            return Some(found);
        }
    }

    if !suffix.is_empty()
        && let Some(path) = scrape_native_preview_pnpm_store(&node_modules, suffix)
    {
        return Some(path);
    }

    None
}

fn find_stable_typescript_native(node_modules: &Path, suffix: &str) -> Option<PathBuf> {
    if let Some(path) = resolve_typescript_platform_package(node_modules, suffix) {
        return Some(path);
    }

    let lib_dir = typescript_platform_package_root(node_modules, suffix).join("lib");
    if let Some(found) = existing_executable(&lib_dir, "tsc") {
        return Some(found);
    }

    scrape_typescript_pnpm_store(node_modules, suffix)
}

fn find_vize_owned_stable_typescript_native(node_modules: &Path, suffix: &str) -> Option<PathBuf> {
    find_stable_typescript_native(&node_modules.join("vize").join("node_modules"), suffix)
}

/// Resolve the TypeScript 7 platform binary through its package manifest.
fn resolve_typescript_platform_package(node_modules: &Path, suffix: &str) -> Option<PathBuf> {
    let package_dir = node_modules.join("typescript");
    let manifest = std::fs::read_to_string(package_dir.join("package.json")).ok()?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest).ok()?;
    if !package_major_at_least(&manifest, 7) {
        return None;
    }
    let platform_package = crate::cstr!("@typescript/typescript-{suffix}");
    if !manifest
        .get("optionalDependencies")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|dependencies| dependencies.contains_key(platform_package.as_str()))
    {
        return None;
    }

    let real_package_dir = package_dir.canonicalize().unwrap_or(package_dir);
    for ancestor in real_package_dir.ancestors() {
        let package_root = typescript_platform_package_root(&ancestor.join("node_modules"), suffix);
        if !package_root.is_dir() {
            continue;
        }
        let lib_dir = package_root.join("lib");
        if let Some(found) = existing_executable(&lib_dir, "tsc") {
            return Some(found);
        }
    }

    None
}

fn package_major_at_least(manifest: &serde_json::Value, minimum: u64) -> bool {
    manifest
        .get("version")
        .and_then(serde_json::Value::as_str)
        .and_then(|version| version.split('.').next())
        .and_then(|major| major.parse::<u64>().ok())
        .is_some_and(|major| major >= minimum)
}

/// Resolve the legacy native-preview platform binary through its manifest.
fn resolve_native_preview_platform_package(node_modules: &Path, suffix: &str) -> Option<PathBuf> {
    let package_dir = node_modules.join("@typescript").join("native-preview");
    let manifest = std::fs::read_to_string(package_dir.join("package.json")).ok()?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest).ok()?;
    let platform_package = crate::cstr!("@typescript/native-preview-{suffix}");
    if !manifest
        .get("optionalDependencies")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|dependencies| dependencies.contains_key(platform_package.as_str()))
    {
        return None;
    }

    let real_package_dir = package_dir.canonicalize().unwrap_or(package_dir);
    for ancestor in real_package_dir.ancestors() {
        let package_root =
            native_preview_platform_package_root(&ancestor.join("node_modules"), suffix);
        if !package_root.is_dir() {
            continue;
        }
        let lib_dir = package_root.join("lib");
        for executable in CORSA_EXECUTABLE_NAMES {
            if let Some(found) = existing_executable(&lib_dir, executable) {
                return Some(found);
            }
        }
    }

    None
}

fn native_preview_platform_package_root(node_modules: &Path, suffix: &str) -> PathBuf {
    node_modules
        .join("@typescript")
        .join(&*crate::cstr!("native-preview-{suffix}"))
}

fn typescript_platform_package_root(node_modules: &Path, suffix: &str) -> PathBuf {
    node_modules
        .join("@typescript")
        .join(&*crate::cstr!("typescript-{suffix}"))
}

/// Legacy fallback: scan the pnpm virtual store for layouts where the platform
/// package exists in the store but is not linked at this `node_modules` level.
fn scrape_native_preview_pnpm_store(node_modules: &Path, suffix: &str) -> Option<PathBuf> {
    let store = node_modules.join(".pnpm");
    let entries = std::fs::read_dir(&store).ok()?;

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("@typescript+native-preview-") || !name.contains(suffix) {
            continue;
        }
        let lib_dir =
            native_preview_platform_package_root(&entry.path().join("node_modules"), suffix)
                .join("lib");
        for executable in CORSA_EXECUTABLE_NAMES {
            if let Some(found) = existing_executable(&lib_dir, executable) {
                return Some(found);
            }
        }
    }

    None
}

fn scrape_typescript_pnpm_store(node_modules: &Path, suffix: &str) -> Option<PathBuf> {
    let store = node_modules.join(".pnpm");
    let entries = std::fs::read_dir(&store).ok()?;

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("@typescript+typescript-") || !name.contains(suffix) {
            continue;
        }
        let lib_dir = typescript_platform_package_root(&entry.path().join("node_modules"), suffix)
            .join("lib");
        if let Some(found) = existing_executable(&lib_dir, "tsc") {
            return Some(found);
        }
    }

    None
}

fn find_node_modules_wrapper(dir: &Path) -> Option<PathBuf> {
    let node_modules = dir.join("node_modules");

    for executable in CORSA_EXECUTABLE_NAMES {
        let candidate = node_modules.join(".bin").join(executable);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    if typescript_package_major_at_least(&node_modules, 7) {
        for candidate in [
            node_modules.join(".bin").join("tsc"),
            node_modules.join("typescript").join("bin").join("tsc"),
        ] {
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    for executable in CORSA_EXECUTABLE_NAMES {
        let candidate = node_modules
            .join("@typescript")
            .join("native-preview")
            .join("bin")
            .join(executable);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

fn typescript_package_major_at_least(node_modules: &Path, minimum: u64) -> bool {
    std::fs::read_to_string(node_modules.join("typescript").join("package.json"))
        .ok()
        .and_then(|manifest| serde_json::from_str::<serde_json::Value>(&manifest).ok())
        .is_some_and(|manifest| package_major_at_least(&manifest, minimum))
}

fn find_in_home_locations() -> Option<PathBuf> {
    const HOME_RELATIVE: [&str; 7] = [
        ".npm-global/bin",
        ".npm/bin",
        ".local/share/pnpm",
        ".volta/bin",
        ".asdf/shims",
        ".local/share/fnm/node-versions/current/bin",
        ".nvm/versions/node/current/bin",
    ];
    const SYSTEM_BIN_DIRS: [&str; 2] = ["/opt/homebrew/bin", "/usr/local/bin"];

    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    for executable in CORSA_EXECUTABLE_NAMES {
        for prefix in HOME_RELATIVE {
            let candidate = home.join(prefix).join(executable);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        for system_dir in SYSTEM_BIN_DIRS {
            let candidate = Path::new(system_dir).join(executable);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    None
}

fn find_in_npm_global_root() -> Option<PathBuf> {
    let output = std::process::Command::new("npm")
        .args(["root", "-g"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let npm_root = String::from_utf8_lossy(&output.stdout);
    let bin_dir = Path::new(npm_root.trim()).parent()?.join("bin");
    for executable in CORSA_EXECUTABLE_NAMES {
        let candidate = bin_dir.join(executable);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

fn find_in_path_lookup() -> Option<PathBuf> {
    for executable in CORSA_EXECUTABLE_NAMES {
        if let Ok(path) = which::which(executable) {
            return Some(path);
        }
    }

    None
}

fn wrapper_project_root(path: &Path) -> Option<&Path> {
    let bin_dir = path.parent()?;
    if bin_dir.file_name()? != ".bin" {
        return None;
    }
    let node_modules = bin_dir.parent()?;
    if node_modules.file_name()? != "node_modules" {
        return None;
    }
    node_modules.parent()
}

fn push_unique(paths: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !paths.contains(&candidate) {
        paths.push(candidate);
    }
}

#[cfg(test)]
mod tests;
