//! Qualify the installed SDK once when a mapper project opens.
//!
//! The snapshot is generated from the native consumer list and TypeScript's
//! module exports, including conditional entry points and private dependencies.
//! Comparing the complete declaration closure avoids parsing SDK types per SFC
//! and prevents ignored scaffold diagnostics from turning a missing type into any.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use vize_canon::virtual_ts::SHARED_VIRTUAL_TYPE_EXPORTS;
use vize_l0::{FxHashMap, String as CompactString, cstr};

#[derive(Default)]
pub(super) struct HelperCapability {
    pub core_types: bool,
    pub virtual_module: bool,
    pub watched_files: Vec<CompactString>,
    pub fingerprint: CompactString,
}

#[derive(Deserialize)]
struct Contract {
    schema: u32,
    consumers: Vec<CompactString>,
    modules: Vec<Module>,
}

#[derive(Deserialize)]
struct Module {
    subpath: CompactString,
    exports: Value,
    files: Vec<Declaration>,
}

#[derive(Deserialize)]
struct Declaration {
    path: CompactString,
    sha256: CompactString,
}

fn contract() -> Option<&'static Contract> {
    static CONTRACT: OnceLock<Option<Contract>> = OnceLock::new();
    CONTRACT
        .get_or_init(|| {
            let contract: Contract = serde_json::from_str(include_str!("helper-abi.json")).ok()?;
            let consumers_match = contract.consumers.len() == SHARED_VIRTUAL_TYPE_EXPORTS.len()
                && SHARED_VIRTUAL_TYPE_EXPORTS.iter().all(|name| {
                    contract
                        .consumers
                        .iter()
                        .any(|consumer| consumer.as_str() == *name)
                });
            (contract.schema == 1 && consumers_match).then_some(contract)
        })
        .as_ref()
}

pub(super) fn installed_helpers(
    config_file_name: &Path,
    compiler_options: &Value,
) -> HelperCapability {
    let mut capability = HelperCapability::default();
    let Some(project_directory) = config_file_name.parent() else {
        return capability;
    };
    let mut fingerprint = Sha256::new();
    if !supports_sdk_resolution(
        config_file_name,
        compiler_options,
        &mut capability.watched_files,
        &mut fingerprint,
    ) {
        capability.fingerprint = cstr!("{:x}", fingerprint.finalize());
        return capability;
    }
    let Some((manifest_path, package_identity)) = locate_node_package(
        project_directory,
        "vize",
        &mut capability.watched_files,
        &mut fingerprint,
    ) else {
        capability.fingerprint = cstr!("{:x}", fingerprint.finalize());
        return capability;
    };
    let Some(package) = manifest_path.parent() else {
        return capability;
    };
    let Some(contract) = contract() else {
        return capability;
    };
    let manifest_bytes = std::fs::read(&manifest_path).ok();
    fingerprint.update(package_identity.to_string_lossy().as_bytes());
    fingerprint.update(manifest_bytes.as_deref().unwrap_or_default());
    // Watch missing declarations too: repairing an incomplete install must
    // invalidate the project settings, rather than retain an earlier fallback.
    let mut declarations = FxHashMap::default();
    for module in &contract.modules {
        for declaration in &module.files {
            declarations
                .entry(declaration.path.as_str())
                .or_insert_with(|| {
                    fingerprint.update(declaration.path.as_bytes());
                    let path = package.join(declaration.path.as_str());
                    capability
                        .watched_files
                        .push(CompactString::from(path.to_string_lossy().as_ref()));
                    match std::fs::read(path) {
                        Ok(contents) => {
                            let digest = Sha256::digest(contents);
                            fingerprint.update(digest);
                            cstr!("{digest:x}") == declaration.sha256
                        }
                        Err(_) => {
                            fingerprint.update(b"missing");
                            false
                        }
                    }
                });
        }
    }
    // TypeScript normally resolves package imports through their real path.
    // A pnpm package's peer dependencies belong to that physical package, not
    // to the project's logical node_modules symlink. preserveSymlinks opts in
    // to the latter authority for both TypeScript and this capability gate.
    let helper_root = if compiler_options.get("preserveSymlinks") == Some(&Value::Bool(true)) {
        package
    } else {
        package_identity.as_path()
    };
    let project_vue = resolve_node_package(
        project_directory,
        "vue",
        &mut capability.watched_files,
        &mut fingerprint,
    );
    let helper_vue = resolve_node_package(
        helper_root,
        "vue",
        &mut capability.watched_files,
        &mut fingerprint,
    );
    let same_vue = project_vue
        .zip(helper_vue)
        .is_some_and(|(project_vue, helper_vue)| project_vue == helper_vue);
    let manifest =
        manifest_bytes.and_then(|contents| serde_json::from_slice::<Value>(&contents).ok());
    let qualified = |subpath: &str| {
        same_vue
            && manifest
                .as_ref()
                .is_some_and(|manifest| manifest.get("typesVersions").is_none())
            && contract.modules.iter().any(|module| {
                module.subpath == subpath
                    && manifest.as_ref().and_then(|manifest| {
                        manifest
                            .get("exports")
                            .and_then(|exports| exports.get(subpath))
                    }) == Some(&module.exports)
                    && module
                        .files
                        .iter()
                        .all(|file| declarations.get(file.path.as_str()) == Some(&true))
            })
    };
    capability.core_types = qualified("./content-mapper-helpers");
    capability.virtual_module =
        capability.core_types && qualified("./virtual-ts") && qualified("./virtual-ts-checker");
    fingerprint.update([
        u8::from(capability.core_types),
        u8::from(capability.virtual_module),
    ]);
    capability.fingerprint = cstr!("{:x}", fingerprint.finalize());
    capability
}

/// Qualify only resolution profiles represented by the generated contract.
/// User aliases must not redirect generated imports to an unchecked module.
fn supports_sdk_resolution(
    config_file_name: &Path,
    compiler_options: &Value,
    watched_files: &mut Vec<CompactString>,
    fingerprint: &mut Sha256,
) -> bool {
    const IMPORTS: &[&str] = &[
        "vize/content-mapper-helpers",
        "vize/virtual-ts",
        "vize/virtual-ts-checker",
    ];
    if compiler_options.get("noResolve") == Some(&Value::Bool(true)) {
        return false;
    }
    if let Some(suffixes) = compiler_options.get("moduleSuffixes")
        && suffixes != &serde_json::json!([""])
    {
        return false;
    }
    if compiler_options.get("paths").is_some() || compiler_options.get("baseUrl").is_some() {
        for specifier in IMPORTS {
            let Some(candidates) = super::super::check::imports_aliases::mapper_override_candidates(
                config_file_name,
                compiler_options,
                specifier,
            ) else {
                return false;
            };
            for candidate in candidates {
                let Some(parent) = candidate.parent() else {
                    return false;
                };
                watched_files.push(CompactString::from(candidate.to_string_lossy().as_ref()));
                watched_files.push(CompactString::from(parent.to_string_lossy().as_ref()));
                fingerprint.update(candidate.to_string_lossy().as_bytes());
                fingerprint.update(b"\0");
                // A missing parent excludes every TypeScript file/extension and
                // package condition. An existing parent remains unqualified;
                // this is a negative proof, not another module resolver.
                match std::fs::metadata(parent) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        fingerprint.update(b"missing-parent\0");
                    }
                    _ => return false,
                }
            }
        }
    }
    true
}

fn resolve_node_package(
    from: &Path,
    package: &str,
    watched_files: &mut Vec<CompactString>,
    fingerprint: &mut Sha256,
) -> Option<PathBuf> {
    locate_node_package(from, package, watched_files, fingerprint).map(|(_, canonical)| canonical)
}

/// Return the logical manifest path and the canonical package directory.
fn locate_node_package(
    from: &Path,
    package: &str,
    watched_files: &mut Vec<CompactString>,
    fingerprint: &mut Sha256,
) -> Option<(PathBuf, PathBuf)> {
    from.ancestors().find_map(|directory| {
        let candidate = directory
            .join("node_modules")
            .join(package)
            .join("package.json");
        watched_files.push(CompactString::from(candidate.to_string_lossy().as_ref()));
        fingerprint.update(candidate.to_string_lossy().as_bytes());
        fingerprint.update(b"\0");
        let Ok(manifest) = std::fs::canonicalize(&candidate) else {
            fingerprint.update(b"missing\0");
            return None;
        };
        // The package directory owns relative type/dependency resolution. A
        // package.json file may itself be a symlink to unrelated metadata.
        let identity = std::fs::canonicalize(candidate.parent()?).ok()?;
        watched_files.push(CompactString::from(manifest.to_string_lossy().as_ref()));
        fingerprint.update(manifest.to_string_lossy().as_bytes());
        fingerprint.update(b"\0");
        fingerprint.update(identity.to_string_lossy().as_bytes());
        fingerprint.update(b"\0");
        if let Ok(contents) = std::fs::read(&manifest) {
            fingerprint.update(Sha256::digest(contents));
        }
        Some((candidate, identity))
    })
}
