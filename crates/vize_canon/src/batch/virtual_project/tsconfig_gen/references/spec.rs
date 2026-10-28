use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_carton::{FxHashMap, FxHashSet};

use super::{
    graph,
    ownership::TsconfigSourceKind,
    parse_jsonc_value, resolve_extended_tsconfig_path,
    wildcard::{TsconfigWildcard, TsconfigWildcardUsage},
};

#[derive(Clone)]
pub(super) struct OwnershipSpec {
    /// `**/*` under the config directory, used when no `include` is inherited.
    default_include: Option<TsconfigWildcard>,
    files: Option<Vec<PathBuf>>,
    includes: Option<Vec<TsconfigWildcard>>,
    excludes: Option<Vec<TsconfigWildcard>>,
    out_dir: Option<Option<TsconfigWildcard>>,
    declaration_dir: Option<Option<TsconfigWildcard>>,
    pub(super) allow_js: Option<bool>,
}

impl OwnershipSpec {
    /// Membership of a path already passed through `graph::normalize_path`.
    pub(super) fn includes_normalized(
        &self,
        source_path: &Path,
        case_sensitive: bool,
        source_kind: TsconfigSourceKind,
    ) -> bool {
        if self.files.as_ref().is_some_and(|files| {
            files
                .iter()
                .any(|file| paths_equal(file, source_path, case_sensitive))
        }) {
            return true;
        }
        if source_kind == TsconfigSourceKind::JavaScript && self.allow_js != Some(true) {
            return false;
        }

        let included = match self.includes.as_ref() {
            Some(includes) => includes
                .iter()
                .any(|include| include.matches(source_path, case_sensitive)),
            None if self.files.is_some() => false,
            None => self
                .default_include
                .as_ref()
                .is_some_and(|include| include.matches(source_path, case_sensitive)),
        };
        if !included {
            return false;
        }

        let excluded = match self.excludes.as_ref() {
            Some(excludes) => excludes
                .iter()
                .any(|exclude| exclude.matches(source_path, case_sensitive)),
            None => self
                .out_dir
                .as_ref()
                .and_then(Option::as_ref)
                .into_iter()
                .chain(self.declaration_dir.as_ref().and_then(Option::as_ref))
                .any(|exclude| exclude.matches(source_path, case_sensitive)),
        };
        !excluded
    }

    fn apply_extended(&mut self, extended: Self) {
        if extended.files.is_some() {
            self.files = extended.files;
        }
        if extended.includes.is_some() {
            self.includes = extended.includes;
        }
        if extended.excludes.is_some() {
            self.excludes = extended.excludes;
        }
        if extended.out_dir.is_some() {
            self.out_dir = extended.out_dir;
        }
        if extended.declaration_dir.is_some() {
            self.declaration_dir = extended.declaration_dir;
        }
        if extended.allow_js.is_some() {
            self.allow_js = extended.allow_js;
        }
    }
}

#[derive(Default)]
pub(super) struct SpecCache {
    active: FxHashSet<PathBuf>,
    completed: FxHashMap<PathBuf, Option<OwnershipSpec>>,
}

impl SpecCache {
    pub(super) fn load(&mut self, config_path: &Path) -> Option<OwnershipSpec> {
        let normalized = graph::normalize_path(config_path);
        if let Some(cached) = self.completed.get(&normalized) {
            return cached.clone();
        }
        if !self.active.insert(normalized.clone()) {
            return None;
        }
        let loaded = self.load_uncached(&normalized);
        self.active.remove(&normalized);
        self.completed.insert(normalized, loaded.clone());
        loaded
    }

    fn load_uncached(&mut self, config_path: &Path) -> Option<OwnershipSpec> {
        let content = std::fs::read_to_string(config_path).ok()?;
        let config = parse_jsonc_value(&content).ok()?;
        let base = config_path.parent().unwrap_or(Path::new("."));
        let mut effective = OwnershipSpec {
            default_include: TsconfigWildcard::new(base, "**/*", TsconfigWildcardUsage::Include),
            files: None,
            includes: None,
            excludes: None,
            out_dir: None,
            declaration_dir: None,
            allow_js: None,
        };

        for parent in extends_entries(&config) {
            if let Some(parent) = resolve_extended_tsconfig_path(config_path, parent)
                .and_then(|path| self.load(&path))
            {
                effective.apply_extended(parent);
            }
        }

        if let Some(files) = string_array(&config, "files") {
            effective.files = Some(
                files
                    .into_iter()
                    .map(|file| graph::normalize_path(&base.join(file)))
                    .collect(),
            );
        }
        if let Some(includes) = string_array(&config, "include") {
            effective.includes = Some(
                includes
                    .into_iter()
                    .filter_map(|include| {
                        TsconfigWildcard::new(base, include, TsconfigWildcardUsage::Include)
                    })
                    .collect(),
            );
        }
        if let Some(excludes) = string_array(&config, "exclude") {
            effective.excludes = Some(
                excludes
                    .into_iter()
                    .filter_map(|exclude| {
                        TsconfigWildcard::new(base, exclude, TsconfigWildcardUsage::Exclude)
                    })
                    .collect(),
            );
        }

        let compiler_options = config.get("compilerOptions").and_then(Value::as_object);
        let own_allow_js = compiler_options.and_then(|options| options.get("allowJs"));
        if config_path
            .file_name()
            .is_some_and(|name| name == "jsconfig.json")
        {
            effective.allow_js = match own_allow_js {
                None => Some(true),
                Some(value) => value.as_bool(),
            };
        } else if let Some(allow_js) = own_allow_js.and_then(Value::as_bool) {
            effective.allow_js = Some(allow_js);
        }
        for (name, target) in [
            ("outDir", &mut effective.out_dir),
            ("declarationDir", &mut effective.declaration_dir),
        ] {
            if let Some(output_dir) = compiler_options
                .and_then(|options| options.get(name))
                .and_then(Value::as_str)
            {
                // `tsc` skips an empty directory; its default exclude is the raw
                // option value.
                *target = Some(
                    (!output_dir.is_empty())
                        .then(|| {
                            TsconfigWildcard::new(base, output_dir, TsconfigWildcardUsage::Exclude)
                        })
                        .flatten(),
                );
            }
        }
        Some(effective)
    }
}

fn extends_entries(config: &Value) -> Vec<&str> {
    match config.get("extends") {
        Some(Value::String(parent)) => vec![parent.as_str()],
        Some(Value::Array(parents)) => parents.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    }
}

fn string_array<'a>(config: &'a Value, name: &str) -> Option<Vec<&'a str>> {
    config
        .get(name)?
        .as_array()
        .map(|entries| entries.iter().filter_map(Value::as_str).collect())
}

fn paths_equal(left: &Path, right: &Path, case_sensitive: bool) -> bool {
    relative_to(left, right, case_sensitive).is_some_and(|path| path.as_os_str().is_empty())
}

/// Both paths are already `graph::normalize_path` results: specs normalize
/// their bases and files once at load, and callers normalize each source once.
/// Normalizing here again canonicalized twice per glob per source.
fn relative_to(path: &Path, base: &Path, case_sensitive: bool) -> Option<PathBuf> {
    let mut path_parts = path.components();
    for base_part in base.components() {
        let path_part = path_parts.next()?;
        if !names_equal(path_part.as_os_str(), base_part.as_os_str(), case_sensitive) {
            return None;
        }
    }
    Some(path_parts.map(|part| part.as_os_str()).collect())
}

fn names_equal(left: &std::ffi::OsStr, right: &std::ffi::OsStr, case_sensitive: bool) -> bool {
    if case_sensitive {
        left == right
    } else {
        left.to_string_lossy().to_lowercase() == right.to_string_lossy().to_lowercase()
    }
}
