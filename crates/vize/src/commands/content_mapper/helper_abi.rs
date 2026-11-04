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
    if !supports_sdk_resolution(project_directory, compiler_options) {
        return capability;
    }
    let Some(package) = project_directory.ancestors().find_map(|directory| {
        let package = directory.join("node_modules/vize");
        package.join("package.json").is_file().then_some(package)
    }) else {
        return capability;
    };
    let manifest_path = package.join("package.json");
    capability.watched_files.push(CompactString::from(
        manifest_path.to_string_lossy().as_ref(),
    ));
    let Some(contract) = contract() else {
        return capability;
    };
    let manifest_bytes = std::fs::read(&manifest_path).ok();
    let mut fingerprint = Sha256::new();
    let package_identity = std::fs::canonicalize(&package).unwrap_or_else(|_| package.clone());
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
    let same_vue = resolve_node_package(project_directory, "vue")
        .zip(resolve_node_package(&package, "vue"))
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
fn supports_sdk_resolution(project_directory: &Path, compiler_options: &Value) -> bool {
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
    if let Some(paths) = compiler_options.get("paths").and_then(Value::as_object)
        && paths.keys().any(|pattern| {
            IMPORTS.iter().any(|name| match pattern.split_once('*') {
                Some((prefix, suffix)) => name.starts_with(prefix) && name.ends_with(suffix),
                None => pattern.as_str() == *name,
            })
        })
    {
        return false;
    }
    if let Some(base_url) = compiler_options.get("baseUrl").and_then(Value::as_str) {
        let base = project_directory.join(base_url);
        if base.join("vize").exists() {
            return false;
        }
    }
    true
}

fn resolve_node_package(from: &Path, package: &str) -> Option<PathBuf> {
    from.ancestors().find_map(|directory| {
        std::fs::canonicalize(
            directory
                .join("node_modules")
                .join(package)
                .join("package.json"),
        )
        .ok()
    })
}
