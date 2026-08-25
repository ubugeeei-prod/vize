use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_s0::{FxHashSet, String};

use super::imports::ImportFileOptions;
use super::path_cache::CanonicalPathCache;
use super::tsconfig_inputs::{parse_jsonc_value, read_extends_entries, resolve_extended_tsconfig};

#[derive(Default)]
pub(super) struct PathAliasResolver {
    aliases: Vec<PathAlias>,
    base_url: Option<PathBuf>,
    module_resolution: Option<String>,
    module: Option<String>,
    custom_conditions: Vec<String>,
    custom_conditions_set: bool,
    paths_set: bool,
}

struct PathAlias {
    prefix: String,
    suffix: String,
    has_wildcard: bool,
    targets: Vec<String>,
    base_dir: PathBuf,
}

impl PathAliasResolver {
    pub(super) fn from_tsconfig(tsconfig_path: Option<&Path>) -> Self {
        let Some(tsconfig_path) = tsconfig_path else {
            return Self::default();
        };
        let mut seen = FxHashSet::default();
        load_aliases(tsconfig_path, &mut seen).unwrap_or_default()
    }

    pub(super) fn resolve(
        &self,
        specifier: &str,
        canonical_paths: &mut CanonicalPathCache,
        options: impl Into<ImportFileOptions>,
        resolve_base: impl Fn(&Path, &mut CanonicalPathCache, ImportFileOptions) -> Option<PathBuf>,
    ) -> Option<PathBuf> {
        let options = options.into();
        for alias in &self.aliases {
            let Some(matched) = alias.match_specifier(specifier) else {
                continue;
            };
            for target in &alias.targets {
                let target = if target.contains('*') {
                    alias.base_dir.join(target.replace('*', matched))
                } else {
                    alias.base_dir.join(target.as_str())
                };
                if let Some(resolved) = resolve_base(&target, canonical_paths, options) {
                    return Some(resolved);
                }
            }
        }
        self.base_url
            .as_ref()
            .and_then(|base_url| resolve_base(&base_url.join(specifier), canonical_paths, options))
    }

    pub(super) fn resolve_with_inputs(
        &self,
        specifier: &str,
        canonical_paths: &mut CanonicalPathCache,
        options: impl Into<ImportFileOptions>,
        mut resolve_base: impl FnMut(
            &Path,
            &mut CanonicalPathCache,
            ImportFileOptions,
        ) -> (Option<PathBuf>, Vec<PathBuf>),
    ) -> (Option<PathBuf>, Vec<PathBuf>) {
        let options = options.into();
        let mut inputs = Vec::new();
        for alias in &self.aliases {
            let Some(matched) = alias.match_specifier(specifier) else {
                continue;
            };
            for target in &alias.targets {
                let target = if target.contains('*') {
                    alias.base_dir.join(target.replace('*', matched))
                } else {
                    alias.base_dir.join(target.as_str())
                };
                let (resolved, consulted) = resolve_base(&target, canonical_paths, options);
                inputs.extend(consulted);
                if resolved.is_some() {
                    return (resolved, inputs);
                }
            }
        }
        if let Some(base_url) = &self.base_url {
            let (resolved, consulted) =
                resolve_base(&base_url.join(specifier), canonical_paths, options);
            inputs.extend(consulted);
            return (resolved, inputs);
        }
        (None, inputs)
    }

    pub(super) fn package_resolution_context(
        &self,
        resolver: &mut vize_canon::PackageRouteResolver,
        importer: &Path,
        occurrence_mode: vize_canon::PackageResolutionMode,
    ) -> (vize_canon::PackageResolutionContext, Vec<PathBuf>) {
        resolver.resolution_context(
            importer,
            occurrence_mode,
            self.module_resolution.as_deref(),
            self.module.as_deref(),
            self.custom_conditions.iter().cloned(),
        )
    }

    fn merge_from(&mut self, parent: Self) {
        if parent.module_resolution.is_some() {
            self.module_resolution = parent.module_resolution;
        }
        if parent.module.is_some() {
            self.module = parent.module;
        }
        if parent.custom_conditions_set {
            self.custom_conditions = parent.custom_conditions;
            self.custom_conditions_set = true;
        }
        if parent.base_url.is_some() {
            self.base_url = parent.base_url;
        }
        if parent.paths_set {
            self.aliases = parent.aliases;
            self.paths_set = true;
        }
    }
}

impl PathAlias {
    fn match_specifier<'a>(&self, specifier: &'a str) -> Option<&'a str> {
        if !self.has_wildcard {
            return (self.prefix == specifier).then_some("");
        }
        specifier
            .strip_prefix(self.prefix.as_str())?
            .strip_suffix(self.suffix.as_str())
    }
}

fn load_aliases(
    tsconfig_path: &Path,
    seen: &mut FxHashSet<PathBuf>,
) -> std::io::Result<PathAliasResolver> {
    let tsconfig_path = tsconfig_path
        .canonicalize()
        .unwrap_or_else(|_| tsconfig_path.to_path_buf());
    if !seen.insert(tsconfig_path.clone()) {
        return Ok(PathAliasResolver::default());
    }

    let content = std::fs::read_to_string(&tsconfig_path)?;
    let value = parse_jsonc_value(&content).unwrap_or(Value::Null);
    let dir = tsconfig_path.parent().unwrap_or(Path::new("."));

    let mut resolver = PathAliasResolver::default();
    for extends in read_extends_entries(&value) {
        if let Some(extended) = resolve_extended_tsconfig(&tsconfig_path, &extends) {
            resolver.merge_from(load_aliases(&extended, seen)?);
        }
    }

    let Some(options) = value.get("compilerOptions").and_then(Value::as_object) else {
        return Ok(resolver);
    };
    if let Some(module_resolution) = options.get("moduleResolution").and_then(Value::as_str) {
        resolver.module_resolution = Some(module_resolution.into());
    }
    if let Some(module) = options.get("module").and_then(Value::as_str) {
        resolver.module = Some(module.into());
    }
    if let Some(custom_conditions) = options.get("customConditions") {
        resolver.custom_conditions_set = true;
        resolver.custom_conditions = custom_conditions
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(String::from)
            .collect();
    }
    if let Some(base_url) = options
        .get("baseUrl")
        .and_then(Value::as_str)
        .map(|base| dir.join(base))
    {
        resolver.base_url = Some(base_url);
    }
    let Some(paths) = options.get("paths").and_then(Value::as_object) else {
        return Ok(resolver);
    };
    resolver.paths_set = true;
    let base_dir = resolver
        .base_url
        .clone()
        .unwrap_or_else(|| dir.to_path_buf());

    resolver.aliases.clear();
    for (pattern, targets) in paths {
        let Some(targets) = targets.as_array() else {
            continue;
        };
        let (prefix, suffix, has_wildcard) = split_pattern(pattern);
        let targets = targets
            .iter()
            .filter_map(Value::as_str)
            .map(String::from)
            .collect::<Vec<_>>();
        if !targets.is_empty() {
            resolver.aliases.push(PathAlias {
                prefix,
                suffix,
                has_wildcard,
                targets,
                base_dir: base_dir.clone(),
            });
        }
    }
    resolver
        .aliases
        .sort_by_key(|alias| std::cmp::Reverse(alias.prefix.len()));
    Ok(resolver)
}

fn split_pattern(pattern: &str) -> (String, String, bool) {
    match pattern.split_once('*') {
        Some((prefix, suffix)) => (prefix.into(), suffix.into(), true),
        None => (pattern.into(), String::default(), false),
    }
}

#[cfg(test)]
#[path = "imports_aliases_tests.rs"]
mod tests;
