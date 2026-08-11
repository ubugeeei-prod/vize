use std::path::{Path, PathBuf};

use glob::{MatchOptions, Pattern};
use serde_json::Value;
use vize_carton::{FxHashMap, FxHashSet, String as CompactString};

use super::{
    graph, implicit_exclude::wildcard_hits_package_folder, ownership::TsconfigSourceKind,
    parse_jsonc_value, resolve_extended_tsconfig_path,
};

#[derive(Clone)]
pub(super) struct OwnershipSpec {
    default_base: PathBuf,
    files: Option<Vec<PathBuf>>,
    includes: Option<Vec<GlobSpec>>,
    excludes: Option<Vec<GlobSpec>>,
    out_dir: Option<Option<GlobSpec>>,
    declaration_dir: Option<Option<GlobSpec>>,
    pub(super) allow_js: Option<bool>,
}

impl OwnershipSpec {
    pub(super) fn includes(
        &self,
        source_path: &Path,
        case_sensitive: bool,
        source_kind: TsconfigSourceKind,
    ) -> bool {
        let source_path = graph::normalize_path(source_path);
        if self.files.as_ref().is_some_and(|files| {
            files
                .iter()
                .any(|file| paths_equal(file, &source_path, case_sensitive))
        }) {
            return true;
        }
        if source_kind == TsconfigSourceKind::JavaScript && self.allow_js != Some(true) {
            return false;
        }

        let included = match self.includes.as_ref() {
            Some(includes) => includes
                .iter()
                .any(|include| include.matches_include(&source_path, case_sensitive)),
            None if self.files.is_some() => false,
            None => GlobSpec::new(&self.default_base, "**/*")
                .is_some_and(|include| include.matches_include(&source_path, case_sensitive)),
        };
        if !included {
            return false;
        }

        let excluded = match self.excludes.as_ref() {
            Some(excludes) => excludes
                .iter()
                .any(|exclude| exclude.matches(&source_path, case_sensitive)),
            None => self
                .out_dir
                .as_ref()
                .and_then(Option::as_ref)
                .into_iter()
                .chain(self.declaration_dir.as_ref().and_then(Option::as_ref))
                .any(|exclude| exclude.matches(&source_path, case_sensitive)),
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
            default_base: graph::normalize_path(base),
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
                    .filter_map(|include| GlobSpec::new(base, include))
                    .collect(),
            );
        }
        if let Some(excludes) = string_array(&config, "exclude") {
            effective.excludes = Some(
                excludes
                    .into_iter()
                    .filter_map(|exclude| GlobSpec::new(base, exclude))
                    .collect(),
            );
        }

        let compiler_options = config.get("compilerOptions").and_then(Value::as_object);
        if let Some(allow_js) = compiler_options
            .and_then(|options| options.get("allowJs"))
            .and_then(Value::as_bool)
        {
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
                *target = Some(GlobSpec::output_dir(base, output_dir));
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

#[derive(Clone)]
struct GlobSpec {
    base: PathBuf,
    pattern: Pattern,
    normalized: CompactString,
}

impl GlobSpec {
    fn new(base: &Path, raw: &str) -> Option<Self> {
        let (base, normalized) = normalize_glob_base(base, raw);
        Pattern::new(normalized.as_str()).ok().map(|pattern| Self {
            base: graph::normalize_path(&base),
            pattern,
            normalized,
        })
    }

    fn output_dir(base: &Path, raw: &str) -> Option<Self> {
        if raw.is_empty() {
            return None;
        }
        let raw = Path::new(raw);
        let resolved = if raw.is_absolute() {
            raw.to_path_buf()
        } else {
            base.join(raw)
        };
        Self::new(&resolved, "**/*")
    }

    fn matches(&self, path: &Path, case_sensitive: bool) -> bool {
        self.matched_relative(path, case_sensitive).is_some()
    }

    fn matches_include(&self, path: &Path, case_sensitive: bool) -> bool {
        self.matched_relative(path, case_sensitive)
            .is_some_and(|relative| {
                !wildcard_hits_package_folder(self.normalized.as_str(), &relative, case_sensitive)
            })
    }

    fn matched_relative(&self, path: &Path, case_sensitive: bool) -> Option<PathBuf> {
        let relative = relative_to(path, &self.base, case_sensitive)?;
        let normalized = relative.to_string_lossy().replace('\\', "/");
        self.pattern
            .matches_with(
                &normalized,
                MatchOptions {
                    case_sensitive,
                    require_literal_separator: true,
                    require_literal_leading_dot: false,
                },
            )
            .then_some(relative)
    }
}

fn normalize_glob_base(base: &Path, raw: &str) -> (PathBuf, CompactString) {
    let mut base = base.to_path_buf();
    let mut pattern = normalize_glob(raw);
    loop {
        if let Some(rest) = pattern.strip_prefix("./") {
            pattern = rest.into();
        } else if let Some(rest) = pattern.strip_prefix("../") {
            base = base.parent().unwrap_or(Path::new(".")).to_path_buf();
            pattern = rest.into();
        } else {
            break;
        }
    }
    (base, pattern)
}

fn normalize_glob(raw: &str) -> CompactString {
    let mut value: CompactString = raw.replace('\\', "/").into();
    if value.is_empty() || value == "." {
        return "**/*".into();
    }
    if !value.contains(['*', '?', '[']) && Path::new(&value).extension().is_none() {
        if !value.ends_with('/') {
            value.push('/');
        }
        value.push_str("**/*");
    }
    value
}

fn paths_equal(left: &Path, right: &Path, case_sensitive: bool) -> bool {
    relative_to(left, right, case_sensitive).is_some_and(|path| path.as_os_str().is_empty())
}

fn relative_to(path: &Path, base: &Path, case_sensitive: bool) -> Option<PathBuf> {
    let path = graph::normalize_path(path);
    let base = graph::normalize_path(base);
    let mut path_parts = path.components();
    for base_part in base.components() {
        let path_part = path_parts.next()?;
        if !names_equal(path_part.as_os_str(), base_part.as_os_str(), case_sensitive) {
            return None;
        }
    }
    Some(path_parts.map(|part| part.as_os_str()).collect())
}

pub(super) fn names_equal(
    left: &std::ffi::OsStr,
    right: &std::ffi::OsStr,
    case_sensitive: bool,
) -> bool {
    if case_sensitive {
        left == right
    } else {
        left.to_string_lossy().to_lowercase() == right.to_string_lossy().to_lowercase()
    }
}
