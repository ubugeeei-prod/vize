//! Solution-style tsconfig handling (#3915).
//!
//! The create-vue default layout ships a references-only shell —
//! `{ "files": [], "references": [{ "path": "./tsconfig.app.json" }, …] }` —
//! with every compiler option, including `paths`, living in the referenced
//! project configs. An anchor that reads only the shell resolves no aliases,
//! so consumers that found no `paths` in the anchored chain retry through the
//! configs the shell references.

use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_carton::{FxHashMap, FxHashSet, String as CompactString};

use super::super::tsconfig_paths::{
    normalize_path_lexically, parse_jsonc_value, resolve_extended_tsconfig_path,
};

/// The project configs referenced by `tsconfig_path`, in declaration order.
///
/// A reference `path` may name a config file or a directory (TypeScript
/// resolves a directory to its `tsconfig.json`); entries that do not resolve
/// to an existing file are dropped. A config without references — or one that
/// cannot be read — yields an empty list.
pub(in super::super) fn referenced_project_configs(tsconfig_path: &Path) -> Vec<PathBuf> {
    let mut configs = Vec::new();
    let mut visited = FxHashSet::default();
    collect_referenced_project_configs(tsconfig_path, &mut visited, &mut configs);
    configs
}

fn collect_referenced_project_configs(
    tsconfig_path: &Path,
    visited: &mut FxHashSet<PathBuf>,
    configs: &mut Vec<PathBuf>,
) {
    let normalized = normalize_path_lexically(tsconfig_path);
    if !visited.insert(normalized.clone()) {
        return;
    }
    let Ok(content) = std::fs::read_to_string(tsconfig_path) else {
        return;
    };
    let Ok(config) = parse_jsonc_value(&content) else {
        return;
    };
    let Some(references) = config.get("references").and_then(Value::as_array) else {
        return;
    };
    let base = tsconfig_path.parent().unwrap_or(Path::new("."));
    for config in references
        .iter()
        .filter_map(|reference| reference.get("path").and_then(Value::as_str))
        .filter_map(|path| {
            let joined = normalize_path_lexically(&base.join(path));
            if joined.is_file() {
                return Some(joined);
            }
            let as_directory = joined.join("tsconfig.json");
            as_directory.is_file().then_some(as_directory)
        })
    {
        if !configs.contains(&config) {
            configs.push(config.clone());
        }
        collect_referenced_project_configs(&config, visited, configs);
    }
}

/// Select the one referenced configured project that owns `source_path`.
/// Ambiguous ownership fails closed to the solution shell; TypeScript can then
/// report/configure the ambiguity instead of Vize choosing a project by order.
pub(in super::super) fn effective_config_for_source(
    tsconfig_path: &Path,
    source_path: &Path,
) -> PathBuf {
    let matching = referenced_project_configs(tsconfig_path)
        .into_iter()
        .filter(|config| config_includes_source(config, source_path))
        .collect::<Vec<_>>();
    if let [config] = matching.as_slice() {
        return config.clone();
    }
    tsconfig_path.to_path_buf()
}

fn config_includes_source(config_path: &Path, source_path: &Path) -> bool {
    let mut load = MembershipLoad::default();
    load.membership(config_path)
        .is_some_and(|membership| membership.includes(source_path))
}

#[derive(Clone)]
struct DeclaredPatterns {
    base: PathBuf,
    values: Vec<CompactString>,
}

impl DeclaredPatterns {
    fn from_config(config: &Value, name: &str, base: &Path) -> Option<Self> {
        let value = config.get(name)?;
        let values = value
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(CompactString::from)
            .collect();
        Some(Self {
            base: base.to_path_buf(),
            values,
        })
    }

    fn matches(&self, source_path: &Path) -> bool {
        self.values.iter().any(|pattern| {
            let absolute = normalize_path_lexically(&self.base.join(pattern.as_str()));
            if !pattern.contains(['*', '?', '[']) {
                return source_path == absolute || source_path.starts_with(&absolute);
            }
            let Ok(relative) = source_path.strip_prefix(&self.base) else {
                return false;
            };
            glob::Pattern::new(pattern.as_str())
                .ok()
                .is_some_and(|pattern| {
                    pattern.matches(&relative.to_string_lossy().replace('\\', "/"))
                })
        })
    }
}

#[derive(Clone)]
struct ProjectMembership {
    default_base: PathBuf,
    files: Option<DeclaredPatterns>,
    include: Option<DeclaredPatterns>,
    exclude: Option<DeclaredPatterns>,
}

impl ProjectMembership {
    fn absorb(&mut self, inherited: Self) {
        if inherited.files.is_some() {
            self.files = inherited.files;
        }
        if inherited.include.is_some() {
            self.include = inherited.include;
        }
        if inherited.exclude.is_some() {
            self.exclude = inherited.exclude;
        }
    }

    fn includes(&self, source_path: &Path) -> bool {
        let source_path = normalize_path_lexically(source_path);
        if self
            .files
            .as_ref()
            .is_some_and(|files| files.matches(&source_path))
        {
            return true;
        }
        let included = match self.include.as_ref() {
            Some(include) => include.matches(&source_path),
            None if self.files.is_some() => false,
            None => source_path.starts_with(&self.default_base),
        };
        if !included {
            return false;
        }
        let default_excluded = source_path
            .strip_prefix(&self.default_base)
            .ok()
            .is_some_and(|relative| {
                relative.components().any(|part| {
                    matches!(
                        part.as_os_str().to_str(),
                        Some("node_modules" | "bower_components" | "jspm_packages")
                    )
                })
            });
        !self
            .exclude
            .as_ref()
            .map_or(default_excluded, |exclude| exclude.matches(&source_path))
    }
}

#[derive(Default)]
struct MembershipLoad {
    active: FxHashSet<PathBuf>,
    completed: FxHashMap<PathBuf, ProjectMembership>,
}

impl MembershipLoad {
    fn membership(&mut self, config_path: &Path) -> Option<ProjectMembership> {
        let normalized = normalize_path_lexically(config_path);
        if let Some(cached) = self.completed.get(&normalized) {
            return Some(cached.clone());
        }
        if !self.active.insert(normalized.clone()) {
            return None;
        }
        let loaded = self.load_config(&normalized);
        self.active.remove(&normalized);
        if let Some(loaded) = loaded.as_ref() {
            self.completed.insert(normalized, loaded.clone());
        }
        loaded
    }

    fn load_config(&mut self, config_path: &Path) -> Option<ProjectMembership> {
        let content = std::fs::read_to_string(config_path).ok()?;
        let config = parse_jsonc_value(&content).ok()?;
        let base = config_path.parent().unwrap_or(Path::new("."));
        let mut effective = ProjectMembership {
            default_base: base.to_path_buf(),
            files: None,
            include: None,
            exclude: None,
        };
        let parents = match config.get("extends") {
            Some(Value::String(parent)) => vec![parent.as_str()],
            Some(Value::Array(parents)) => parents.iter().filter_map(Value::as_str).collect(),
            _ => Vec::new(),
        };
        for parent in parents {
            if let Some(parent) = resolve_extended_tsconfig_path(config_path, parent)
                .and_then(|path| self.membership(&path))
            {
                effective.absorb(parent);
            }
        }
        for name in ["files", "include", "exclude"] {
            if config.get(name).is_none() {
                continue;
            }
            let value = DeclaredPatterns::from_config(&config, name, base);
            match name {
                "files" => effective.files = value,
                "include" => effective.include = value,
                "exclude" => effective.exclude = value,
                _ => unreachable!(),
            }
        }
        Some(effective)
    }
}

#[cfg(test)]
#[path = "references/tests.rs"]
mod tests;
