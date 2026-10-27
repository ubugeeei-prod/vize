//! Load and flatten one `extends` chain of `vueCompilerOptions`.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::batch::error::{CorsaError, CorsaResult};

use super::super::super::tsconfig_paths::{
    normalize_path_lexically, parse_jsonc_value, resolve_extended_tsconfig_path_with_dependencies,
    resolve_extended_tsconfig_path_with_inputs,
};
use super::super::compiler_inputs::{CompilerInputs, read_jsonc};
use super::ChainLoad;

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
pub(in super::super) fn load_vue_compiler_options(
    tsconfig_path: Option<&Path>,
) -> CorsaResult<Map<std::string::String, Value>> {
    load_vue_compiler_options_with_inputs(tsconfig_path, None)
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
pub(in super::super) fn load_vue_compiler_options_with_inputs(
    tsconfig_path: Option<&Path>,
    inputs: Option<&CompilerInputs>,
) -> CorsaResult<Map<std::string::String, Value>> {
    let Some(tsconfig_path) = tsconfig_path else {
        return Ok(Map::new());
    };
    let mut load = ChainLoad {
        inputs,
        ..Default::default()
    };
    load_vue_compiler_options_inner(tsconfig_path, &mut load)
}

/// Resolve Vue options through the existing chain authority, retaining all file probes.
#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
pub(in super::super::super) fn snapshot_vue_compiler_options(
    path: &Path,
) -> CorsaResult<(Map<std::string::String, Value>, Vec<PathBuf>)> {
    let mut load = ChainLoad {
        dependencies: Some(Vec::new()),
        ..Default::default()
    };
    let options = load_vue_compiler_options_inner(path, &mut load)?;
    let mut dependencies = load.dependencies.unwrap_or_default();
    dependencies.sort();
    dependencies.dedup();
    Ok((options, dependencies))
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
fn load_vue_compiler_options_inner(
    tsconfig_path: &Path,
    load: &mut ChainLoad<'_>,
) -> CorsaResult<Map<std::string::String, Value>> {
    if let Some(inputs) = load.inputs {
        inputs.observe(tsconfig_path);
    }
    if let Some(dependencies) = &mut load.dependencies {
        dependencies.push(tsconfig_path.to_path_buf());
    }
    if !tsconfig_path.exists() {
        if load.dependencies.is_some() {
            return Err(CorsaError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                vize_carton::cstr!("Cannot read tsconfig '{}'", tsconfig_path.display()),
            )));
        }
        return Ok(Map::new());
    }
    let normalized = normalize_path_lexically(tsconfig_path);
    if let Some(cached) = load.completed.get(&normalized) {
        return Ok(cached.clone());
    }
    if !load.active.insert(normalized.clone()) {
        load.cycle_cut = true;
        return Ok(Map::new());
    }
    let enclosing_cycle_cut = std::mem::replace(&mut load.cycle_cut, false);
    let flattened = merge_extended_vue_compiler_options(&normalized, load);
    load.active.remove(&normalized);
    let chain_cycle_cut = load.cycle_cut;
    load.cycle_cut = enclosing_cycle_cut || chain_cycle_cut;
    if !chain_cycle_cut && let Ok(flattened) = &flattened {
        load.completed.insert(normalized, flattened.clone());
    }
    flattened
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
fn merge_extended_vue_compiler_options(
    normalized: &Path,
    load: &mut ChainLoad<'_>,
) -> CorsaResult<Map<std::string::String, Value>> {
    let config = if load.dependencies.is_some() {
        let content = std::fs::read_to_string(normalized)?;
        parse_jsonc_value(&content).map_err(|error| {
            CorsaError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                vize_carton::cstr!("Invalid tsconfig '{}': {error}", normalized.display()),
            ))
        })?
    } else {
        read_jsonc(normalized, load.inputs)?
    };
    let current = config
        .get("vueCompilerOptions")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let mut inherited = Map::new();
    match config.get("extends") {
        Some(Value::String(extends)) => {
            if let Some(parent_path) = resolve_parent(normalized, extends, load)? {
                inherited = load_vue_compiler_options_inner(&parent_path, load)?;
            }
        }
        Some(Value::Array(entries)) => {
            for extends in entries.iter().filter_map(Value::as_str) {
                if let Some(parent_path) = resolve_parent(normalized, extends, load)? {
                    inherited.extend(load_vue_compiler_options_inner(&parent_path, load)?);
                }
            }
        }
        _ => {}
    }
    inherited.extend(current);
    Ok(inherited)
}

fn resolve_parent(
    path: &Path,
    extends: &str,
    load: &mut ChainLoad<'_>,
) -> CorsaResult<Option<PathBuf>> {
    match &mut load.dependencies {
        Some(dependencies) => {
            let parent =
                resolve_extended_tsconfig_path_with_dependencies(path, extends, dependencies)
                    .ok_or_else(|| {
                        CorsaError::Io(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            vize_carton::cstr!(
                                "Cannot resolve extended tsconfig '{extends}' from '{}'",
                                path.display()
                            ),
                        ))
                    })?;
            Ok(Some(parent))
        }
        None => Ok(resolve_extended_tsconfig_path_with_inputs(
            path,
            extends,
            load.inputs,
        )),
    }
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
pub(super) fn check_unknown_props_enabled(options: &Map<std::string::String, Value>) -> bool {
    if let Some(value) = options.get("checkUnknownProps").and_then(Value::as_bool) {
        return value;
    }
    options
        .get("strictTemplates")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}
