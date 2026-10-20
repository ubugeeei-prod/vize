//! Package-owned native TypeScript runtime layouts and wrapper discovery.

use std::path::{Path, PathBuf};

use super::{CORSA_EXECUTABLE_NAMES, Candidate, existing_executable, platform_suffix};

pub(super) fn find_node_modules_native(dir: &Path) -> Option<Candidate> {
    let node_modules = dir.join("node_modules");
    let suffix = platform_suffix();

    if !suffix.is_empty() {
        if let Some(path) = find_stable_typescript_native(&node_modules, suffix) {
            return Some(Candidate::Native(path));
        }
        if let Some(path) = find_vize_owned_stable_typescript_native(&node_modules, suffix) {
            return Some(Candidate::Native(path));
        }

        // Node-style resolution: let the meta package's manifest tell us which
        // platform package to look for, then resolve it like `require` would.
        if let Some(path) = resolve_native_preview_platform_package(&node_modules, suffix) {
            return Some(Candidate::Legacy(path));
        }

        let lib_dir = native_preview_platform_package_root(&node_modules, suffix).join("lib");
        for executable in CORSA_EXECUTABLE_NAMES {
            if let Some(found) = existing_executable(&lib_dir, executable) {
                return Some(Candidate::Legacy(found));
            }
        }
    }

    let meta_lib_dir = node_modules
        .join("@typescript")
        .join("native-preview")
        .join("lib");
    for executable in CORSA_EXECUTABLE_NAMES {
        if let Some(found) = existing_executable(&meta_lib_dir, executable) {
            return Some(Candidate::Legacy(found));
        }
    }

    if !suffix.is_empty()
        && let Some(path) = scrape_native_preview_pnpm_store(&node_modules, suffix)
    {
        return Some(Candidate::Legacy(path));
    }

    if let Some(path) = find_node_modules_tsrs_native(dir) {
        return Some(Candidate::Wrapper(path));
    }

    None
}

/// Resolve the native binary selected by the `tsrs` launcher package.
///
/// This avoids making a standalone Vize binary depend on `node` merely to run
/// the JavaScript launcher. `tsrs` is kept behind the TypeScript/Corsa runtime
/// tiers unless it is explicitly selected through `TSRS_PATH` or
/// `typeChecker.tsrsPath`.
pub(super) fn find_node_modules_tsrs_native(dir: &Path) -> Option<PathBuf> {
    let node_modules = dir.join("node_modules");
    let package_dir = node_modules.join("tsrs");
    let manifest = std::fs::read_to_string(package_dir.join("package.json")).ok()?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest).ok()?;
    let platform_package = tsrs_platform_package()?;
    if !manifest
        .get("optionalDependencies")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|dependencies| dependencies.contains_key(platform_package))
    {
        return None;
    }

    let real_package_dir = package_dir.canonicalize().unwrap_or(package_dir);
    for ancestor in real_package_dir.ancestors() {
        let candidate = ancestor
            .join("node_modules")
            .join(platform_package)
            .join("tsrs");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

pub(super) fn tsrs_platform_package() -> Option<&'static str> {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Some("@tsrs/darwin-arm64")
    } else if cfg!(all(
        target_os = "linux",
        target_arch = "aarch64",
        target_env = "musl"
    )) {
        Some("@tsrs/linux-arm64-musl")
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        Some("@tsrs/linux-arm64-gnu")
    } else if cfg!(all(
        target_os = "linux",
        target_arch = "x86_64",
        target_env = "musl"
    )) {
        Some("@tsrs/linux-x64-musl")
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Some("@tsrs/linux-x64-gnu")
    } else {
        None
    }
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

pub(super) fn find_node_modules_wrapper(dir: &Path) -> Option<PathBuf> {
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
