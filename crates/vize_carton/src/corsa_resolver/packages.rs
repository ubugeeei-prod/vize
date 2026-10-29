//! Package-owned native TypeScript runtime layouts and wrapper discovery.

use std::path::{Path, PathBuf};

use super::{CORSA_EXECUTABLE_NAMES, Candidate, existing_executable, platform_suffix};

/// A declared runtime package that is installed but incomplete or mismatched.
#[derive(Debug)]
pub(super) struct InvalidNativePackage {
    pub(super) package: PathBuf,
}

pub(super) fn find_node_modules_native(
    dir: &Path,
) -> Result<Option<Candidate>, InvalidNativePackage> {
    let project = read_manifest(dir).unwrap_or_default();
    let suffix = platform_suffix();

    if !suffix.is_empty() {
        if let Some(path) = find_stable_typescript_native(dir, suffix, &project)? {
            return Ok(Some(Candidate::Native(path)));
        }

        if let Some(path) = find_declared_native_preview(dir, suffix, &project)? {
            return Ok(Some(Candidate::Legacy(path)));
        }
    }

    if let Some(path) = find_declared_tsrs(dir, &project)? {
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
fn resolve_tsrs_package(package_dir: &Path) -> Option<PathBuf> {
    let manifest = std::fs::read_to_string(package_dir.join("package.json")).ok()?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest).ok()?;
    if manifest.get("name").and_then(serde_json::Value::as_str) != Some("tsrs") {
        return None;
    }
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
    project: &serde_json::Value,
) -> Result<Option<PathBuf>, InvalidNativePackage> {
    let platform_package = crate::cstr!("@typescript/typescript-{suffix}");
    let mut names = declared_dependency_names(project);
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
            let invalid = || InvalidNativePackage {
                package: package_dir.clone(),
            };
            let manifest = read_manifest(&package_dir).ok_or_else(invalid)?;
            if manifest.get("name").and_then(serde_json::Value::as_str) != Some(name)
                || !package_major_at_least(&manifest, 7)
            {
                return Err(invalid());
            }
            return existing_executable(&package_dir.join("lib"), "tsc")
                .map(Some)
                .ok_or_else(invalid);
        }
        let required = declaration_targets(project, name, "typescript");
        if let Some(path) =
            resolve_typescript_platform_package(&package_dir, platform_package.as_str(), required)?
        {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

fn find_declared_native_preview(
    dir: &Path,
    suffix: &str,
    project: &serde_json::Value,
) -> Result<Option<PathBuf>, InvalidNativePackage> {
    let canonical = "@typescript/native-preview";
    let platform = crate::cstr!("@typescript/native-preview-{suffix}");

    for name in declared_dependency_names(project) {
        let declared_runtime = declaration_targets(project, name, canonical)
            || declaration_targets(project, name, platform.as_str());
        let Some(package_dir) = nearest_package(dir, name) else {
            if declared_runtime {
                return Err(InvalidNativePackage {
                    package: dir.join("node_modules").join(name),
                });
            }
            continue;
        };
        let Some(manifest) = read_manifest(&package_dir) else {
            if declared_runtime {
                return Err(InvalidNativePackage {
                    package: package_dir,
                });
            }
            continue;
        };
        match manifest.get("name").and_then(serde_json::Value::as_str) {
            Some(actual) if actual == platform.as_str() => {
                let invalid = || InvalidNativePackage {
                    package: package_dir.clone(),
                };
                if !package_major_at_least(&manifest, 7) {
                    return Err(invalid());
                }
                return runtime_executable(&package_dir.join("lib"))
                    .map(Some)
                    .ok_or_else(invalid);
            }
            Some(actual) if actual == canonical => {
                return resolve_native_preview_package(&package_dir, suffix).map(Some);
            }
            _ if declared_runtime => {
                return Err(InvalidNativePackage {
                    package: package_dir,
                });
            }
            _ => {}
        }
    }

    Ok(None)
}

fn resolve_native_preview_package(
    package_dir: &Path,
    suffix: &str,
) -> Result<PathBuf, InvalidNativePackage> {
    let invalid = || InvalidNativePackage {
        package: package_dir.to_path_buf(),
    };
    let manifest = read_manifest(package_dir).ok_or_else(invalid)?;
    if manifest.get("name").and_then(serde_json::Value::as_str)
        != Some("@typescript/native-preview")
    {
        return Err(invalid());
    }

    let platform = crate::cstr!("@typescript/native-preview-{suffix}");
    if let Some(version) = manifest
        .get("optionalDependencies")
        .and_then(|dependencies| dependencies.get(platform.as_str()))
        .and_then(serde_json::Value::as_str)
    {
        let real_package_dir = package_dir.canonicalize().map_err(|_| invalid())?;
        let platform_dir =
            nearest_package(&real_package_dir, platform.as_str()).ok_or_else(invalid)?;
        let installed = read_manifest(&platform_dir).ok_or_else(invalid)?;
        if installed.get("name").and_then(serde_json::Value::as_str) != Some(platform.as_str())
            || installed.get("version").and_then(serde_json::Value::as_str) != Some(version)
        {
            return Err(invalid());
        }
        return runtime_executable(&platform_dir.join("lib")).ok_or_else(invalid);
    }

    runtime_executable(&package_dir.join("lib")).ok_or_else(invalid)
}

fn find_declared_tsrs(
    dir: &Path,
    project: &serde_json::Value,
) -> Result<Option<PathBuf>, InvalidNativePackage> {
    for name in declared_dependency_names(project) {
        let declared_runtime = declaration_targets(project, name, "tsrs");
        let Some(package_dir) = nearest_package(dir, name) else {
            if declared_runtime {
                return Err(InvalidNativePackage {
                    package: dir.join("node_modules").join(name),
                });
            }
            continue;
        };
        let actual = read_manifest(&package_dir)
            .and_then(|manifest| manifest.get("name")?.as_str().map(str::to_owned));
        if actual.as_deref() == Some("tsrs") {
            return resolve_tsrs_package(&package_dir).map(Some).ok_or_else(|| {
                InvalidNativePackage {
                    package: package_dir,
                }
            });
        }
        if declared_runtime {
            return Err(InvalidNativePackage {
                package: package_dir,
            });
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

/// An npm name is `name` or `@scope/name`. Separators and prefixes of any
/// host (`\`, `C:`) are rejected, so a key cannot leave `node_modules`.
pub(super) fn valid_package_name(name: &str) -> bool {
    let segment =
        |part: &str| !part.is_empty() && !part.starts_with('.') && !part.contains(['\\', ':']);
    match name.split_once('/') {
        None => !name.starts_with('@') && segment(name),
        Some((scope, package)) => scope
            .strip_prefix('@')
            .is_some_and(|scope| segment(scope) && segment(package) && !package.contains('/')),
    }
}

fn read_manifest(package: &Path) -> Option<serde_json::Value> {
    serde_json::from_str(&std::fs::read_to_string(package.join("package.json")).ok()?).ok()
}

fn declared_dependency_names(project: &serde_json::Value) -> Vec<&str> {
    let mut names = Vec::new();
    for section in ["dependencies", "devDependencies", "optionalDependencies"] {
        if let Some(dependencies) = project.get(section).and_then(serde_json::Value::as_object) {
            names.extend(
                dependencies
                    .keys()
                    .filter(|name| valid_package_name(name))
                    .map(String::as_str),
            );
        }
    }
    names.sort_unstable();
    names.dedup();
    names
}

fn dependency_spec<'a>(project: &'a serde_json::Value, name: &str) -> Option<&'a str> {
    ["dependencies", "devDependencies", "optionalDependencies"]
        .iter()
        .find_map(|section| project.get(section)?.get(name)?.as_str())
}

fn declaration_targets(project: &serde_json::Value, name: &str, target: &str) -> bool {
    if name == target {
        return true;
    }
    dependency_spec(project, name).is_some_and(|spec| {
        spec.strip_prefix("npm:").is_some_and(|aliased| {
            aliased == target
                || aliased
                    .strip_prefix(target)
                    .is_some_and(|version| version.starts_with('@'))
        })
    })
}

/// Follow the installed meta package, including aliases and pnpm symlinks.
/// The first linked platform package owns Node's lookup: a mismatched version
/// cannot be skipped in favour of an unrelated ancestor's runtime.
fn resolve_typescript_platform_package(
    package_dir: &Path,
    platform: &str,
    required: bool,
) -> Result<Option<PathBuf>, InvalidNativePackage> {
    let invalid = || InvalidNativePackage {
        package: package_dir.to_path_buf(),
    };
    let Some(manifest) = read_manifest(package_dir) else {
        return if required { Err(invalid()) } else { Ok(None) };
    };
    if manifest.get("name").and_then(serde_json::Value::as_str) != Some("typescript") {
        return if required { Err(invalid()) } else { Ok(None) };
    }
    let version = manifest
        .get("version")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(invalid)?;
    let major = version
        .split('.')
        .next()
        .and_then(|major| major.parse::<u64>().ok())
        .ok_or_else(invalid)?;
    if major < 7 {
        return Ok(None);
    }
    if manifest
        .get("optionalDependencies")
        .and_then(|deps| deps.get(platform))
        .and_then(serde_json::Value::as_str)
        != Some(version)
    {
        return Err(invalid());
    }
    let real_package_dir = package_dir.canonicalize().map_err(|_| invalid())?;
    let package_root = nearest_package(&real_package_dir, platform).ok_or_else(invalid)?;
    let installed = read_manifest(&package_root).ok_or_else(invalid)?;
    if installed.get("name").and_then(serde_json::Value::as_str) != Some(platform)
        || installed.get("version").and_then(serde_json::Value::as_str) != Some(version)
    {
        return Err(invalid());
    }
    existing_executable(&package_root.join("lib"), "tsc")
        .map(Some)
        .ok_or_else(invalid)
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
        value.as_str().is_some_and(|relative| {
            inside_package(relative)
                && package
                    .join(relative)
                    .canonicalize()
                    .is_ok_and(|path| path == target)
        })
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

/// A regular-file `.bin` shim (Windows, pnpm) names its owner only by bin
/// name. Follow the one sibling package that declares it; an ambiguous or
/// foreign owner keeps the shim.
pub(super) fn resolve_bin_shim(shim: &Path) -> Option<PathBuf> {
    let file_name = shim.file_name()?.to_str()?;
    let name = [".cmd", ".ps1", ".exe", ".bunx"]
        .iter()
        .find_map(|extension| file_name.strip_suffix(extension))
        .unwrap_or(file_name);
    let node_modules = shim.parent()?.parent()?;
    let mut owners = sibling_packages(node_modules)
        .into_iter()
        .filter_map(|package| declared_bin(&package, name));
    let target = owners.next()?;
    if owners.next().is_some() {
        return None;
    }
    resolve_owned_launcher(&target.canonicalize().ok()?)
}

fn sibling_packages(node_modules: &Path) -> Vec<PathBuf> {
    let mut packages = Vec::new();
    for entry in std::fs::read_dir(node_modules)
        .into_iter()
        .flatten()
        .flatten()
    {
        let path = entry.path();
        match entry.file_name().to_str() {
            Some(name) if name.starts_with('.') => {}
            Some(name) if name.starts_with('@') => packages.extend(
                std::fs::read_dir(&path)
                    .into_iter()
                    .flatten()
                    .flatten()
                    .map(|entry| entry.path()),
            ),
            Some(_) => packages.push(path),
            None => {}
        }
    }
    packages
}

/// The target `package` declares for bin `name`, kept inside the package.
fn declared_bin(package: &Path, name: &str) -> Option<PathBuf> {
    let manifest = read_manifest(package)?;
    let relative = match manifest.get("bin")? {
        serde_json::Value::String(relative) => {
            let package_name = manifest.get("name")?.as_str()?;
            (package_name.rsplit('/').next() == Some(name)).then_some(relative.as_str())?
        }
        bins => bins.get(name)?.as_str()?,
    };
    inside_package(relative).then(|| package.join(relative))
}

fn inside_package(relative: &str) -> bool {
    Path::new(relative).components().all(|part| {
        matches!(
            part,
            std::path::Component::Normal(_) | std::path::Component::CurDir
        )
    })
}

fn package_major_at_least(manifest: &serde_json::Value, minimum: u64) -> bool {
    manifest
        .get("version")
        .and_then(serde_json::Value::as_str)
        .and_then(|version| version.split('.').next())
        .and_then(|major| major.parse::<u64>().ok())
        .is_some_and(|major| major >= minimum)
}

fn runtime_executable(lib_dir: &Path) -> Option<PathBuf> {
    CORSA_EXECUTABLE_NAMES
        .iter()
        .find_map(|name| existing_executable(lib_dir, name))
}

pub(super) fn find_node_modules_wrapper(dir: &Path) -> Option<PathBuf> {
    let project = read_manifest(dir)?;
    let names = declared_dependency_names(&project);
    for executable in CORSA_EXECUTABLE_NAMES {
        let mut candidates = names
            .iter()
            .filter_map(|name| nearest_package(dir, name))
            .filter(|package| declared_bin(package, executable).is_some())
            .filter_map(|package| {
                let node_modules = package
                    .ancestors()
                    .skip(1)
                    .find(|path| path.file_name().is_some_and(|name| name == "node_modules"))?;
                let wrapper = node_modules.join(".bin").join(executable);
                wrapper.exists().then_some(wrapper)
            });
        let Some(candidate) = candidates.next() else {
            continue;
        };
        if candidates.next().is_none() {
            return Some(candidate);
        }
    }

    None
}
