//! Package-owned native TypeScript runtime layouts and wrapper discovery.

use std::path::{Path, PathBuf};

use super::{CORSA_EXECUTABLE_NAMES, Candidate, existing_executable, platform_suffix};

#[derive(Debug)]
pub(super) struct InvalidNativePackage;

pub(super) fn find_node_modules_native(
    dir: &Path,
) -> Result<Option<Candidate>, InvalidNativePackage> {
    let node_modules = dir.join("node_modules");
    let suffix = platform_suffix();

    if !suffix.is_empty() {
        if let Some(path) = find_stable_typescript_native(dir, suffix)? {
            return Ok(Some(Candidate::Native(path)));
        }

        // Node-style resolution: let the meta package's manifest tell us which
        // platform package to look for, then resolve it like `require` would.
        if let Some(path) = resolve_native_preview_platform_package(&node_modules, suffix) {
            return Ok(Some(Candidate::Legacy(path)));
        }

        let lib_dir = native_preview_platform_package_root(&node_modules, suffix).join("lib");
        for executable in CORSA_EXECUTABLE_NAMES {
            if let Some(found) = existing_executable(&lib_dir, executable) {
                return Ok(Some(Candidate::Legacy(found)));
            }
        }
    }

    let meta_lib_dir = node_modules
        .join("@typescript")
        .join("native-preview")
        .join("lib");
    for executable in CORSA_EXECUTABLE_NAMES {
        if let Some(found) = existing_executable(&meta_lib_dir, executable) {
            return Ok(Some(Candidate::Legacy(found)));
        }
    }

    if let Some(path) = find_node_modules_tsrs_native(dir) {
        return Ok(Some(Candidate::Wrapper(path)));
    }

    Ok(None)
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
    resolve_tsrs_package(&package_dir)
}

fn resolve_tsrs_package(package_dir: &Path) -> Option<PathBuf> {
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

    let real_package_dir = package_dir.canonicalize().ok()?;
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

fn find_stable_typescript_native(
    dir: &Path,
    suffix: &str,
) -> Result<Option<PathBuf>, InvalidNativePackage> {
    let platform_package = crate::cstr!("@typescript/typescript-{suffix}");
    let project = read_manifest(dir).unwrap_or_default();
    let mut names = Vec::new();
    for section in ["dependencies", "devDependencies", "optionalDependencies"] {
        if let Some(dependencies) = project.get(section).and_then(serde_json::Value::as_object) {
            names.extend(
                dependencies
                    .keys()
                    .filter(|name| valid_package_name(name))
                    .map(|name| name.as_str()),
            );
        }
    }
    names.sort_unstable_by_key(|name| {
        let tier = if *name == "typescript" {
            0
        } else if *name == platform_package.as_str() {
            2
        } else {
            1
        };
        (tier, *name)
    });
    names.dedup();
    for name in names {
        let Some(package_dir) = nearest_package(dir, name) else {
            continue;
        };
        if name == platform_package.as_str() {
            let manifest = read_manifest(&package_dir).ok_or(InvalidNativePackage)?;
            if manifest.get("name").and_then(serde_json::Value::as_str) != Some(name)
                || !package_major_at_least(&manifest, 7)
            {
                return Err(InvalidNativePackage);
            }
            return existing_executable(&package_dir.join("lib"), "tsc")
                .map(Some)
                .ok_or(InvalidNativePackage);
        }
        let required = name == "typescript"
            || ["dependencies", "devDependencies", "optionalDependencies"]
                .iter()
                .any(|section| {
                    project
                        .get(section)
                        .and_then(|deps| deps.get(name))
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|value| value.starts_with("npm:typescript@"))
                });
        if let Some(path) =
            resolve_typescript_platform_package(&package_dir, platform_package.as_str(), required)?
        {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

/// A present but invalid nearest slot must not reveal another ancestor's copy.
fn nearest_package(dir: &Path, name: &str) -> Option<PathBuf> {
    dir.ancestors()
        .filter(|ancestor| {
            ancestor
                .file_name()
                .is_none_or(|name| name != "node_modules")
        })
        .map(|ancestor| ancestor.join("node_modules").join(name))
        .find(|path| match std::fs::symlink_metadata(path) {
            Ok(_) => true,
            Err(error) => error.kind() != std::io::ErrorKind::NotFound,
        })
}

fn valid_package_name(name: &str) -> bool {
    !Path::new(name).is_absolute()
        && !name.starts_with('.')
        && name
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

fn read_manifest(package: &Path) -> Option<serde_json::Value> {
    serde_json::from_str(&std::fs::read_to_string(package.join("package.json")).ok()?).ok()
}

/// Follow the installed meta package, including aliases and pnpm symlinks.
/// The first linked platform package owns Node's lookup: a mismatched version
/// cannot be skipped in favour of an unrelated ancestor's runtime.
fn resolve_typescript_platform_package(
    package_dir: &Path,
    platform: &str,
    required: bool,
) -> Result<Option<PathBuf>, InvalidNativePackage> {
    let Some(manifest) = read_manifest(package_dir) else {
        return if required {
            Err(InvalidNativePackage)
        } else {
            Ok(None)
        };
    };
    if manifest.get("name").and_then(serde_json::Value::as_str) != Some("typescript") {
        return if required {
            Err(InvalidNativePackage)
        } else {
            Ok(None)
        };
    }
    if !package_major_at_least(&manifest, 7) {
        return Ok(None);
    }
    let version = manifest
        .get("version")
        .and_then(serde_json::Value::as_str)
        .ok_or(InvalidNativePackage)?;
    if manifest
        .get("optionalDependencies")
        .and_then(|deps| deps.get(platform))
        .and_then(serde_json::Value::as_str)
        != Some(version)
    {
        return Err(InvalidNativePackage);
    }
    let real_package_dir = package_dir
        .canonicalize()
        .map_err(|_| InvalidNativePackage)?;
    let package_root = nearest_package(&real_package_dir, platform).ok_or(InvalidNativePackage)?;
    let installed = read_manifest(&package_root).ok_or(InvalidNativePackage)?;
    if installed.get("name").and_then(serde_json::Value::as_str) != Some(platform)
        || installed.get("version").and_then(serde_json::Value::as_str) != Some(version)
    {
        return Err(InvalidNativePackage);
    }
    existing_executable(&package_root.join("lib"), "tsc")
        .map(Some)
        .ok_or(InvalidNativePackage)
}

/// Map only a bin entry owned by the selected canonical target's package.
pub(super) fn resolve_owned_launcher(target: &Path) -> Option<PathBuf> {
    let package = target
        .ancestors()
        .skip(1)
        .find(|path| path.join("package.json").is_file())?;
    let manifest = read_manifest(package)?;
    let bin = manifest.get("bin")?;
    let owns = |value: &serde_json::Value| {
        let Some(relative) = value.as_str() else {
            return false;
        };
        let path = Path::new(relative);
        path.components().all(|part| {
            matches!(
                part,
                std::path::Component::Normal(_) | std::path::Component::CurDir
            )
        }) && package
            .join(path)
            .canonicalize()
            .is_ok_and(|path| path == target)
    };
    if !owns(bin) && !bin.as_object().is_some_and(|bins| bins.values().any(owns)) {
        return None;
    }
    match manifest.get("name").and_then(serde_json::Value::as_str)? {
        "typescript" => resolve_typescript_platform_package(
            package,
            &crate::cstr!("@typescript/typescript-{}", platform_suffix()),
            true,
        )
        .ok()
        .flatten(),
        "tsrs" => resolve_tsrs_package(package),
        _ => None,
    }
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

pub(super) fn find_node_modules_wrapper(dir: &Path) -> Option<PathBuf> {
    let node_modules = dir.join("node_modules");

    for executable in CORSA_EXECUTABLE_NAMES {
        let candidate = node_modules.join(".bin").join(executable);
        if candidate.exists() {
            return Some(candidate);
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
