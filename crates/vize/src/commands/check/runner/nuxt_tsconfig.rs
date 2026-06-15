//! Nuxt path-alias fallback `tsconfig` synthesis for the `check` runner.
//!
//! When Nuxt auto-imports contribute path aliases, the runner writes a wrapper
//! `tsconfig` under `node_modules/.vize/cli` that extends the project config and
//! rebases inherited `paths` targets relative to the wrapper directory.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::{Map, Value};
use vize_carton::{FxHashSet, String};

use super::JsonObject;
use crate::commands::check::tsconfig_inputs::{
    parse_jsonc_value, read_extends_entries, resolve_extended_tsconfig,
};

pub(super) fn resolve_checker_tsconfig_path(
    program_tsconfig_path: Option<&Path>,
    project_root: &Path,
    nuxt_path_aliases: &[super::super::nuxt::NuxtPathAlias],
) -> Result<Option<PathBuf>, std::io::Error> {
    if nuxt_path_aliases.is_empty() {
        return Ok(program_tsconfig_path.map(Path::to_path_buf));
    }

    write_nuxt_fallback_tsconfig(program_tsconfig_path, project_root, nuxt_path_aliases).map(Some)
}

pub(super) fn write_nuxt_fallback_tsconfig(
    program_tsconfig_path: Option<&Path>,
    project_root: &Path,
    nuxt_path_aliases: &[super::super::nuxt::NuxtPathAlias],
) -> Result<PathBuf, std::io::Error> {
    let wrapper_dir = project_root.join("node_modules/.vize/cli");
    fs::create_dir_all(&wrapper_dir)?;

    let mut paths = if let Some(tsconfig_path) = program_tsconfig_path {
        load_tsconfig_paths_for_wrapper(tsconfig_path, &wrapper_dir)?
    } else {
        Map::new()
    };
    for alias in nuxt_path_aliases {
        paths
            .entry(alias.pattern.as_str().to_owned())
            .or_insert_with(|| {
                Value::Array(
                    alias
                        .targets
                        .iter()
                        .map(|target| {
                            Value::String(
                                rebase_tsconfig_path_target(
                                    &wrapper_dir,
                                    project_root,
                                    target.as_str(),
                                )
                                .into(),
                            )
                        })
                        .collect(),
                )
            });
    }

    let mut compiler_options = Map::new();
    compiler_options.insert("paths".into(), Value::Object(paths));

    let mut config = Map::new();
    if let Some(tsconfig_path) = program_tsconfig_path {
        config.insert(
            "extends".into(),
            Value::String(tsconfig_path.to_string_lossy().into_owned()),
        );
    }
    config.insert("compilerOptions".into(), Value::Object(compiler_options));

    let wrapper_path = wrapper_dir.join("tsconfig.nuxt-fallback.json");
    let content =
        serde_json::to_vec_pretty(&Value::Object(config)).map_err(std::io::Error::other)?;
    fs::write(&wrapper_path, content)?;
    Ok(wrapper_path)
}

fn load_tsconfig_paths_for_wrapper(
    tsconfig_path: &Path,
    wrapper_dir: &Path,
) -> Result<JsonObject, std::io::Error> {
    let mut seen = FxHashSet::default();
    load_tsconfig_paths_for_wrapper_inner(tsconfig_path, wrapper_dir, &mut seen)
}

fn load_tsconfig_paths_for_wrapper_inner(
    tsconfig_path: &Path,
    wrapper_dir: &Path,
    seen: &mut FxHashSet<PathBuf>,
) -> Result<JsonObject, std::io::Error> {
    let resolved = vize_carton::path::canonicalize_non_verbatim(tsconfig_path);
    if !seen.insert(resolved.clone()) {
        return Ok(Map::new());
    }

    let content = fs::read_to_string(&resolved)?;
    let value = parse_jsonc_value(&content).unwrap_or(Value::Null);
    let mut merged = Map::new();

    for extends in read_extends_entries(&value) {
        let Some(extended_path) = resolve_extended_tsconfig(&resolved, &extends) else {
            continue;
        };
        let extended = load_tsconfig_paths_for_wrapper_inner(&extended_path, wrapper_dir, seen)?;
        merged.extend(extended);
    }

    let Some(paths) = value
        .get("compilerOptions")
        .and_then(Value::as_object)
        .and_then(|compiler_options| compiler_options.get("paths"))
        .and_then(Value::as_object)
    else {
        return Ok(merged);
    };

    let base_dir = resolved.parent().unwrap_or(Path::new("."));
    for (alias, targets) in paths {
        merged.insert(
            alias.clone(),
            rebase_tsconfig_paths_value(wrapper_dir, base_dir, targets),
        );
    }

    Ok(merged)
}

fn rebase_tsconfig_paths_value(wrapper_dir: &Path, source_base_dir: &Path, value: &Value) -> Value {
    let Some(targets) = value.as_array() else {
        return value.clone();
    };

    Value::Array(
        targets
            .iter()
            .map(|target| {
                target.as_str().map_or_else(
                    || target.clone(),
                    |target| {
                        Value::String(
                            rebase_tsconfig_path_target(wrapper_dir, source_base_dir, target)
                                .into(),
                        )
                    },
                )
            })
            .collect(),
    )
}

fn rebase_tsconfig_path_target(wrapper_dir: &Path, source_base_dir: &Path, target: &str) -> String {
    let target_path = Path::new(target);
    let target_path = if target_path.is_absolute() {
        target_path.to_path_buf()
    } else {
        source_base_dir.join(target_path)
    };
    let target_path = normalize_path_lexically(&target_path);
    let wrapper_dir = normalize_path_lexically(wrapper_dir);
    let rebased = diff_paths(&target_path, &wrapper_dir).unwrap_or(target_path);
    path_to_tsconfig_target(&rebased)
}

fn path_to_tsconfig_target(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").into()
}

fn normalize_path_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component.as_os_str());
                }
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

fn diff_paths(target: &Path, base: &Path) -> Option<PathBuf> {
    use std::path::Component;

    if target.is_absolute() != base.is_absolute() {
        if target.is_absolute() {
            return Some(target.to_path_buf());
        }
        return None;
    }

    let target_components = target.components().collect::<Vec<_>>();
    let base_components = base.components().collect::<Vec<_>>();
    let mut common = 0;
    while common < target_components.len()
        && common < base_components.len()
        && target_components[common] == base_components[common]
    {
        common += 1;
    }

    if matches!(
        (target_components.first(), base_components.first()),
        (Some(Component::Prefix(_)), Some(Component::Prefix(_)))
    ) && common == 0
    {
        return None;
    }

    let mut relative = PathBuf::new();
    for component in &base_components[common..] {
        match component {
            Component::Normal(_) | Component::CurDir | Component::ParentDir => {
                relative.push("..");
            }
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    for component in &target_components[common..] {
        match component {
            Component::Normal(value) => relative.push(value),
            Component::ParentDir => relative.push(".."),
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    if relative.as_os_str().is_empty() {
        relative.push(".");
    }
    Some(relative)
}
