//! `tsconfig.json`-driven default input collection for `vize check`.
//!
//! When users run `vize check` without explicit paths, we should follow the
//! project's configured `files` / `include` / `exclude` fields instead of
//! recursively scanning every TypeScript file under the working directory.

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use glob::{MatchOptions, Pattern};
use ignore::WalkBuilder;
use serde_json::Value;
use vize_carton::{FxHashSet, profile, profiler::global_profiler};

const TARGET_DIR: &str = "target";
const NODE_MODULES_DIR: &str = "node_modules";
const VIZE_CACHE_DIR: &str = ".vize";

#[derive(Debug, Clone, Default)]
struct TsconfigInputSpec {
    files: Vec<RelativePathSpec>,
    includes: Vec<GlobSpec>,
    excludes: Vec<GlobSpec>,
    has_files: bool,
    has_includes: bool,
    has_excludes: bool,
}

impl TsconfigInputSpec {
    fn apply_extended(&mut self, extended: Self) {
        if extended.has_files {
            self.files = extended.files;
            self.has_files = true;
        }
        if extended.has_includes {
            self.includes = extended.includes;
            self.has_includes = true;
        }
        if extended.has_excludes {
            self.excludes = extended.excludes;
            self.has_excludes = true;
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TsconfigDeclarationOptions {
    pub(crate) declaration_dir: Option<PathBuf>,
    pub(crate) out_dir: Option<PathBuf>,
    pub(crate) declaration_map: Option<bool>,
}

impl TsconfigDeclarationOptions {
    fn apply_extended(&mut self, extended: Self) {
        if extended.declaration_dir.is_some() {
            self.declaration_dir = extended.declaration_dir;
        }
        if extended.out_dir.is_some() {
            self.out_dir = extended.out_dir;
        }
        if extended.declaration_map.is_some() {
            self.declaration_map = extended.declaration_map;
        }
    }

    pub(crate) fn output_dir(&self) -> Option<&Path> {
        self.declaration_dir.as_deref().or(self.out_dir.as_deref())
    }
}

#[derive(Debug, Clone)]
struct RelativePathSpec {
    base_dir: PathBuf,
    value: std::string::String,
}

impl RelativePathSpec {
    fn new(base_dir: &Path, value: &str) -> Self {
        Self {
            base_dir: base_dir.to_path_buf(),
            value: value.replace('\\', "/"),
        }
    }

    fn resolve(&self) -> PathBuf {
        self.base_dir.join(&self.value)
    }
}

#[derive(Debug, Clone)]
struct GlobSpec {
    base_dir: PathBuf,
    pattern: Pattern,
}

impl GlobSpec {
    fn new(base_dir: &Path, value: &str) -> Option<Self> {
        let normalized = normalize_tsconfig_glob(value);
        Pattern::new(&normalized).ok().map(|pattern| Self {
            base_dir: base_dir.to_path_buf(),
            pattern,
        })
    }

    fn matches(&self, path: &Path) -> bool {
        let Ok(relative) = path.strip_prefix(&self.base_dir) else {
            return false;
        };
        let normalized = normalize_path_separators(relative);
        self.pattern.matches_with(&normalized, glob_match_options())
    }
}

pub(crate) fn collect_default_check_files(
    project_root: &Path,
    tsconfig_path: Option<&Path>,
) -> Vec<PathBuf> {
    let Some(tsconfig_path) = tsconfig_path else {
        return collect_supported_files(project_root, &[], &[]);
    };

    let spec = load_tsconfig_inputs(tsconfig_path).unwrap_or_default();
    let mut files = Vec::new();
    let mut seen = FxHashSet::default();

    for file in spec.files {
        let resolved = normalize_input_path(&file.resolve());
        if resolved.starts_with(project_root)
            && resolved.is_file()
            && is_supported_check_file(&resolved)
            && seen.insert(resolved.clone())
        {
            files.push(resolved);
        }
    }

    let default_base_dir = tsconfig_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| project_root.to_path_buf());

    let includes = if !spec.has_includes && !spec.has_files && files.is_empty() {
        GlobSpec::new(&default_base_dir, "**/*")
            .into_iter()
            .collect::<Vec<_>>()
    } else {
        spec.includes
    };

    let excludes = if !spec.has_excludes {
        default_exclude_specs(&default_base_dir)
    } else {
        spec.excludes
    };

    if !includes.is_empty() {
        let collected = collect_supported_files(project_root, &includes, &excludes);
        for path in collected {
            if seen.insert(path.clone()) {
                files.push(path);
            }
        }
    }

    files.sort();
    files
}

/// Collect ambient declaration (`.d.ts`) files that belong to the tsconfig
/// "program" so their global types stay in scope when only a subset of files is
/// checked explicitly (e.g. `vize check src/App.vue`).
///
/// Ambient declarations (`declare global`, top-level `declare const`) are not
/// pulled in by imports, so the explicit-path collector drops them and `tsgo`
/// then reports false `TS2304` errors for genuinely global names. This mirrors
/// `tsc`, which always loads the declaration files matched by `files`/`include`
/// regardless of which entry files are requested.
///
/// Module-shim declaration files (`declare module "vue"`, `declare module
/// "*.css"`) are deliberately excluded: forcing them in as program roots makes
/// their ambient external-module declarations shadow the real package types
/// (a `declare module "vue"` block erases `vue`'s real exports). Those files are
/// already reached through the imports that reference them, so dropping them
/// from the ambient set is safe.
pub(crate) fn collect_ambient_declaration_files(
    project_root: &Path,
    tsconfig_path: Option<&Path>,
) -> Vec<PathBuf> {
    collect_default_check_files(project_root, tsconfig_path)
        .into_iter()
        .filter(|path| is_declaration_file(path))
        .filter(|path| match fs::read_to_string(path) {
            Ok(content) => !declares_ambient_module(&content),
            Err(_) => false,
        })
        .collect()
}

fn is_declaration_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
}

/// Returns `true` when the file declares an ambient *external* module, i.e.
/// `declare module "<specifier>"` with a quoted specifier. Such declarations
/// shadow real package types when the file is loaded as a program root, so
/// these files are excluded from the ambient-globals set. A namespace-style
/// `declare module Foo {}` (no quotes) is a plain global and does not match.
fn declares_ambient_module(content: &str) -> bool {
    const NEEDLE: &str = "declare module";
    content.match_indices(NEEDLE).any(|(index, _)| {
        let preceded_by_boundary = content[..index]
            .chars()
            .next_back()
            .is_none_or(|ch| !ch.is_alphanumeric() && ch != '_' && ch != '$');
        preceded_by_boundary
            && content[index + NEEDLE.len()..]
                .chars()
                .find(|ch| !ch.is_whitespace())
                .is_some_and(|ch| ch == '"' || ch == '\'')
    })
}

pub(crate) fn load_tsconfig_declaration_options(
    tsconfig_path: &Path,
) -> TsconfigDeclarationOptions {
    let mut seen = FxHashSet::default();
    load_tsconfig_declaration_options_inner(tsconfig_path, &mut seen).unwrap_or_default()
}

fn collect_supported_files(
    root: &Path,
    includes: &[GlobSpec],
    excludes: &[GlobSpec],
) -> Vec<PathBuf> {
    let skip_generated = should_skip_generated_for_root(root);
    let normalized_root = normalize_input_path(root);
    let walker = WalkBuilder::new(root)
        .standard_filters(true)
        .hidden(true)
        .build_parallel();

    let collected = std::sync::Mutex::new(Vec::<PathBuf>::new());
    walker.run(|| {
        let collected = &collected;
        let normalized_root = normalized_root.clone();
        Box::new(move |entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file()
                    && is_supported_check_file(path)
                    && (!skip_generated || !is_generated_path(path))
                    && matches_tsconfig_patterns(path, includes, excludes)
                    && let Ok(mut collected) = collected.lock()
                {
                    collected.push(normalize_walked_path(root, &normalized_root, path));
                }
            }
            ignore::WalkState::Continue
        })
    });

    let Ok(mut collected) = collected.into_inner() else {
        return Vec::new();
    };
    collected.sort();
    collected.dedup();
    collected
}

fn matches_tsconfig_patterns(path: &Path, includes: &[GlobSpec], excludes: &[GlobSpec]) -> bool {
    if !includes.is_empty() && !includes.iter().any(|glob| glob.matches(path)) {
        return false;
    }

    !excludes.iter().any(|glob| glob.matches(path))
}

fn load_tsconfig_inputs(tsconfig_path: &Path) -> Option<TsconfigInputSpec> {
    let mut seen = FxHashSet::default();
    load_tsconfig_inputs_inner(tsconfig_path, &mut seen).ok()
}

fn load_tsconfig_inputs_inner(
    tsconfig_path: &Path,
    seen: &mut FxHashSet<PathBuf>,
) -> Result<TsconfigInputSpec, std::io::Error> {
    let resolved = normalize_input_path(tsconfig_path);
    if !seen.insert(resolved.clone()) {
        return Ok(TsconfigInputSpec::default());
    }

    let content = tracked_read_to_string(&resolved)?;
    let value = parse_jsonc_value(&content).unwrap_or(Value::Null);
    let dir = resolved.parent().unwrap_or(Path::new("."));

    let mut merged = TsconfigInputSpec::default();
    for extends in read_extends_entries(&value) {
        let Some(extends_path) = resolve_extended_tsconfig(&resolved, &extends) else {
            continue;
        };
        let extended = load_tsconfig_inputs_inner(&extends_path, seen)?;
        merged.apply_extended(extended);
    }

    if let Some(files) = read_string_array(&value, "files") {
        merged.has_files = true;
        merged.files = files
            .into_iter()
            .map(|value| RelativePathSpec::new(dir, &value))
            .collect();
    }

    if let Some(includes) = read_string_array(&value, "include") {
        merged.has_includes = true;
        merged.includes = includes
            .into_iter()
            .filter_map(|value| GlobSpec::new(dir, &value))
            .collect();
    }

    if let Some(excludes) = read_string_array(&value, "exclude") {
        merged.has_excludes = true;
        merged.excludes = excludes
            .into_iter()
            .filter_map(|value| GlobSpec::new(dir, &value))
            .collect();
    }

    Ok(merged)
}

fn load_tsconfig_declaration_options_inner(
    tsconfig_path: &Path,
    seen: &mut FxHashSet<PathBuf>,
) -> Result<TsconfigDeclarationOptions, std::io::Error> {
    let resolved = normalize_input_path(tsconfig_path);
    if !seen.insert(resolved.clone()) {
        return Ok(TsconfigDeclarationOptions::default());
    }

    let content = tracked_read_to_string(&resolved)?;
    let value = parse_jsonc_value(&content).unwrap_or(Value::Null);
    let dir = resolved.parent().unwrap_or(Path::new("."));

    let mut merged = TsconfigDeclarationOptions::default();
    for extends in read_extends_entries(&value) {
        let Some(extends_path) = resolve_extended_tsconfig(&resolved, &extends) else {
            continue;
        };
        let extended = load_tsconfig_declaration_options_inner(&extends_path, seen)?;
        merged.apply_extended(extended);
    }

    let Some(compiler_options) = value.get("compilerOptions").and_then(Value::as_object) else {
        return Ok(merged);
    };

    if let Some(declaration_dir) = compiler_options
        .get("declarationDir")
        .and_then(Value::as_str)
    {
        merged.declaration_dir = Some(resolve_tsconfig_path_option(dir, declaration_dir));
    }
    if let Some(out_dir) = compiler_options.get("outDir").and_then(Value::as_str) {
        merged.out_dir = Some(resolve_tsconfig_path_option(dir, out_dir));
    }
    if let Some(declaration_map) = compiler_options
        .get("declarationMap")
        .and_then(Value::as_bool)
    {
        merged.declaration_map = Some(declaration_map);
    }

    Ok(merged)
}

fn resolve_extended_tsconfig(tsconfig_path: &Path, extends: &str) -> Option<PathBuf> {
    let base_dir = tsconfig_path.parent().unwrap_or(Path::new("."));
    let mut candidates = Vec::new();

    if Path::new(extends).is_absolute() || extends.starts_with('.') {
        push_tsconfig_candidates(
            &mut candidates,
            if Path::new(extends).is_absolute() {
                PathBuf::from(extends)
            } else {
                base_dir.join(extends)
            },
        );
    } else {
        push_node_modules_tsconfig_candidates(&mut candidates, base_dir, extends);
    }

    candidates.into_iter().find(|candidate| candidate.is_file())
}

fn resolve_tsconfig_path_option(base_dir: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    }
}

fn push_node_modules_tsconfig_candidates(
    candidates: &mut Vec<PathBuf>,
    base_dir: &Path,
    extends: &str,
) {
    let mut current = Some(base_dir);
    while let Some(dir) = current {
        let node_modules = dir.join("node_modules");
        if let Some((package, subpath)) = split_package_specifier(extends) {
            let package_root = node_modules.join(package);
            if let Some(subpath) = subpath {
                push_tsconfig_candidates(candidates, package_root.join(subpath));
            } else {
                push_package_json_tsconfig_candidates(candidates, &package_root);
                candidates.push(package_root.join("tsconfig.json"));
            }
        } else {
            push_tsconfig_candidates(candidates, node_modules.join(extends));
        }
        current = dir.parent();
    }
}

fn split_package_specifier(extends: &str) -> Option<(&str, Option<&str>)> {
    let mut parts = extends.split('/');
    let first = parts.next()?;
    if first.is_empty() {
        return None;
    }

    if first.starts_with('@') {
        let name = parts.next()?;
        if name.is_empty() {
            return None;
        }
        let package_len = first.len() + 1 + name.len();
        let subpath = extends
            .get(package_len + 1..)
            .filter(|value| !value.is_empty());
        return Some((&extends[..package_len], subpath));
    }

    let subpath = extends
        .get(first.len() + 1..)
        .filter(|value| !value.is_empty());
    Some((first, subpath))
}

fn push_package_json_tsconfig_candidates(candidates: &mut Vec<PathBuf>, package_root: &Path) {
    let package_json_path = package_root.join("package.json");
    let Some(tsconfig) = tracked_read_to_string(&package_json_path)
        .ok()
        .and_then(|content| parse_jsonc_value(&content).ok())
        .and_then(|value| {
            value
                .get("tsconfig")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
    else {
        return;
    };

    push_tsconfig_candidates(candidates, package_root.join(tsconfig));
}

fn tracked_read_to_string(path: &Path) -> Result<std::string::String, std::io::Error> {
    match profile!("cli.check.tsconfig.read", fs::read_to_string(path)) {
        Ok(content) => {
            global_profiler().record_fs_read_to_string(content.len());
            Ok(content)
        }
        Err(error) => {
            global_profiler().record_fs_read_to_string_failure();
            Err(error)
        }
    }
}

fn push_tsconfig_candidates(candidates: &mut Vec<PathBuf>, base: PathBuf) {
    candidates.push(base.clone());
    if base.extension().is_none() {
        candidates.push(base.with_extension("json"));
        candidates.push(base.join("tsconfig.json"));
    }
}

fn read_string_array(value: &Value, key: &str) -> Option<Vec<std::string::String>> {
    value.get(key).and_then(Value::as_array).map(|items| {
        items
            .iter()
            .filter_map(|item| item.as_str().map(std::string::String::from))
            .collect()
    })
}

fn read_extends_entries(value: &Value) -> Vec<std::string::String> {
    match value.get("extends") {
        Some(Value::String(extends)) => vec![extends.clone()],
        Some(Value::Array(extends)) => extends
            .iter()
            .filter_map(|item| item.as_str().map(std::string::String::from))
            .collect(),
        _ => Vec::new(),
    }
}

fn normalize_tsconfig_glob(value: &str) -> std::string::String {
    let mut normalized = value.replace('\\', "/");
    if normalized.is_empty() {
        normalized.push_str("**/*");
        return normalized;
    }

    if normalized == "." {
        normalized.clear();
        normalized.push_str("**/*");
        return normalized;
    }

    if normalized.contains(['*', '?', '[']) {
        return normalized;
    }

    let has_extension = Path::new(&normalized).extension().is_some();
    if has_extension {
        return normalized;
    }

    if !normalized.ends_with('/') {
        normalized.push('/');
    }
    normalized.push_str("**/*");
    normalized
}

fn default_exclude_specs(base_dir: &Path) -> Vec<GlobSpec> {
    ["node_modules", "bower_components", "jspm_packages"]
        .into_iter()
        .filter_map(|value| GlobSpec::new(base_dir, value))
        .collect()
}

fn normalize_path_separators(path: &Path) -> std::string::String {
    path.to_string_lossy().replace('\\', "/")
}

fn normalize_input_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn normalize_walked_path(root: &Path, normalized_root: &Path, path: &Path) -> PathBuf {
    // Avoid a canonicalize syscall per walked file; normalize the root once.
    path.strip_prefix(root)
        .map(|relative| normalized_root.join(relative))
        .unwrap_or_else(|_| normalize_input_path(path))
}

fn should_skip_generated_for_root(root: &Path) -> bool {
    !path_is_generated_root(root)
}

fn is_generated_path(path: &Path) -> bool {
    let mut previous = None;
    path.components().any(|component| {
        let Some(name) = component.as_os_str().to_str() else {
            previous = None;
            return false;
        };
        let generated = is_generated_component(previous, name);
        previous = Some(name);
        generated
    })
}

fn path_is_generated_root(path: &Path) -> bool {
    let mut previous = None;
    for component in path.components() {
        let Some(name) = component.as_os_str().to_str() else {
            previous = None;
            continue;
        };
        if is_generated_component(previous, name) {
            return true;
        }
        previous = Some(name);
    }
    false
}

fn is_generated_component(previous: Option<&str>, name: &str) -> bool {
    name == TARGET_DIR || (previous == Some(NODE_MODULES_DIR) && name == VIZE_CACHE_DIR)
}

fn is_supported_check_file(path: &Path) -> bool {
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
    {
        return true;
    }

    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "vue" | "ts" | "tsx" | "mts" | "cts"))
}

fn glob_match_options() -> MatchOptions {
    MatchOptions {
        case_sensitive: !cfg!(windows),
        require_literal_separator: true,
        require_literal_leading_dot: false,
    }
}

fn parse_jsonc_value(content: &str) -> Result<Value, serde_json::Error> {
    let stripped = strip_json_comments(content);
    let normalized = strip_trailing_commas(&stripped);
    serde_json::from_str(&normalized)
}

fn strip_json_comments(content: &str) -> std::string::String {
    let mut output = std::string::String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    let mut line_comment = false;
    let mut block_comment = false;

    while let Some(ch) = chars.next() {
        if line_comment {
            if ch == '\n' {
                line_comment = false;
                output.push('\n');
            }
            continue;
        }

        if block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                let _ = chars.next();
                block_comment = false;
            } else if ch == '\n' {
                output.push('\n');
            }
            continue;
        }

        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            output.push(ch);
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'/') {
            let _ = chars.next();
            line_comment = true;
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'*') {
            let _ = chars.next();
            block_comment = true;
            continue;
        }

        output.push(ch);
    }

    output
}

fn strip_trailing_commas(content: &str) -> std::string::String {
    let mut output = std::string::String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let mut index = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    while index < chars.len() {
        let ch = chars[index];
        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            index += 1;
            continue;
        }

        if ch == '"' {
            in_string = true;
            output.push(ch);
            index += 1;
            continue;
        }

        if ch == ',' {
            let mut lookahead = index + 1;
            while lookahead < chars.len() && chars[lookahead].is_whitespace() {
                lookahead += 1;
            }
            if lookahead < chars.len() && matches!(chars[lookahead], '}' | ']') {
                index += 1;
                continue;
            }
        }

        output.push(ch);
        index += 1;
    }

    output
}

#[cfg(test)]
mod tests {
    use super::{
        collect_ambient_declaration_files, collect_default_check_files,
        load_tsconfig_declaration_options, resolve_extended_tsconfig,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use vize_carton::cstr;

    fn unique_case_dir(name: &str) -> PathBuf {
        static NEXT_CASE_ID: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("vize-tests")
            .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
    }

    #[test]
    fn default_collection_respects_include_and_exclude() {
        let case_dir = unique_case_dir("tsconfig-default");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(case_dir.join("src/generated")).unwrap();
        fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
        fs::write(case_dir.join("src/main.ts"), "export const ok = true").unwrap();
        fs::write(
            case_dir.join("src/generated/skip.ts"),
            "export const skip = true",
        )
        .unwrap();
        fs::write(case_dir.join("vite.config.ts"), "export default {}").unwrap();
        fs::write(
            case_dir.join("tsconfig.json"),
            r#"{
  "include": ["src/**/*.ts", "src/**/*.vue"],
  "exclude": ["src/generated"]
}"#,
        )
        .unwrap();

        let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|path| path.ends_with("src/App.vue")));
        assert!(files.iter().any(|path| path.ends_with("src/main.ts")));
        assert!(!files.iter().any(|path| path.ends_with("vite.config.ts")));
        assert!(
            !files
                .iter()
                .any(|path| path.ends_with("src/generated/skip.ts"))
        );

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn default_collection_inherits_extended_include() {
        let case_dir = unique_case_dir("tsconfig-extends");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(case_dir.join("src")).unwrap();
        fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
        fs::write(case_dir.join("vite.config.ts"), "export default {}").unwrap();
        fs::write(
            case_dir.join("tsconfig.base.json"),
            r#"{
  "include": ["src/**/*.vue"]
}"#,
        )
        .unwrap();
        fs::write(
            case_dir.join("tsconfig.json"),
            r#"{
  "extends": "./tsconfig.base.json"
}"#,
        )
        .unwrap();

        let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

        assert_eq!(files, vec![case_dir.join("src/App.vue")]);

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn declaration_options_inherit_extends_and_use_config_relative_paths() {
        let case_dir = unique_case_dir("tsconfig-declaration-options");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(case_dir.join("configs")).unwrap();
        fs::write(
            case_dir.join("configs/base.json"),
            r#"{
  "compilerOptions": {
    "declarationDir": "base-types",
    "outDir": "base-dist",
    "declarationMap": true
  }
}"#,
        )
        .unwrap();
        fs::write(
            case_dir.join("tsconfig.json"),
            r#"{
  "extends": "./configs/base.json",
  "compilerOptions": {
    "outDir": "dist",
    "declarationMap": false
  }
}"#,
        )
        .unwrap();

        let options = load_tsconfig_declaration_options(&case_dir.join("tsconfig.json"));

        assert_eq!(
            options.declaration_dir,
            Some(case_dir.join("configs/base-types"))
        );
        assert_eq!(options.out_dir, Some(case_dir.join("dist")));
        assert_eq!(options.declaration_map, Some(false));
        assert_eq!(
            options.output_dir(),
            Some(case_dir.join("configs/base-types").as_path())
        );

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn default_collection_applies_extends_array_in_order() {
        let case_dir = unique_case_dir("tsconfig-extends-array");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(case_dir.join("src/one")).unwrap();
        fs::create_dir_all(case_dir.join("src/two")).unwrap();
        fs::write(case_dir.join("src/one/One.vue"), "<template />").unwrap();
        fs::write(case_dir.join("src/two/App.vue"), "<template />").unwrap();
        fs::write(case_dir.join("src/two/Skip.vue"), "<template />").unwrap();
        fs::write(
            case_dir.join("tsconfig.one.json"),
            r#"{
  "include": ["src/one/**/*.vue"],
  "exclude": ["src/two/Skip.vue"]
}"#,
        )
        .unwrap();
        fs::write(
            case_dir.join("tsconfig.two.json"),
            r#"{
  "include": ["src/two/**/*.vue"]
}"#,
        )
        .unwrap();
        fs::write(
            case_dir.join("tsconfig.json"),
            r#"{
  "extends": ["./tsconfig.one.json", "./tsconfig.two.json"]
}"#,
        )
        .unwrap();

        let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

        assert_eq!(files, vec![case_dir.join("src/two/App.vue")]);

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn extended_config_resolution_finds_ancestor_node_modules() {
        let case_dir = unique_case_dir("tsconfig-package-extends");
        let _ = fs::remove_dir_all(&case_dir);
        let app_dir = case_dir.join("packages/app");
        let package_dir = case_dir.join("node_modules/@scope/tsconfig");
        fs::create_dir_all(&app_dir).unwrap();
        fs::create_dir_all(&package_dir).unwrap();
        fs::write(app_dir.join("tsconfig.json"), "{}").unwrap();
        fs::write(
            package_dir.join("tsconfig.vue.json"),
            r#"{
  "compilerOptions": {
    "strict": true
  }
}"#,
        )
        .unwrap();

        let resolved = resolve_extended_tsconfig(
            &app_dir.join("tsconfig.json"),
            "@scope/tsconfig/tsconfig.vue.json",
        );

        assert_eq!(resolved, Some(package_dir.join("tsconfig.vue.json")));

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn extended_config_resolution_uses_package_json_tsconfig_field() {
        let case_dir = unique_case_dir("tsconfig-package-json-field");
        let _ = fs::remove_dir_all(&case_dir);
        let app_dir = case_dir.join("packages/app");
        let package_dir = case_dir.join("node_modules/@scope/tsconfig");
        fs::create_dir_all(app_dir.join("src")).unwrap();
        fs::create_dir_all(package_dir.join("configs")).unwrap();
        fs::write(app_dir.join("tsconfig.json"), "{}").unwrap();
        fs::write(
            package_dir.join("package.json"),
            r#"{
  "name": "@scope/tsconfig",
  "tsconfig": "configs/vue.json"
}"#,
        )
        .unwrap();
        fs::write(
            package_dir.join("configs/vue.json"),
            r#"{
  "compilerOptions": {
    "strict": true
  }
}"#,
        )
        .unwrap();
        fs::write(package_dir.join("tsconfig.json"), "{}").unwrap();

        let resolved = resolve_extended_tsconfig(&app_dir.join("tsconfig.json"), "@scope/tsconfig");

        assert_eq!(resolved, Some(package_dir.join("configs/vue.json")));

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn ambient_declaration_collection_keeps_only_dts_within_include() {
        let case_dir = unique_case_dir("tsconfig-ambient-dts");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(case_dir.join("src/@types")).unwrap();
        fs::write(
            case_dir.join("src/@types/globals.d.ts"),
            "export {};\ndeclare global { type GlobalTabType = 'a' | 'b'; }\n",
        )
        .unwrap();
        fs::write(case_dir.join("src/env.d.ts"), "declare const X: string;").unwrap();
        fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
        fs::write(case_dir.join("src/main.ts"), "export const ok = true").unwrap();
        fs::write(case_dir.join("outside.d.ts"), "declare const Y: string;").unwrap();
        fs::write(
            case_dir.join("tsconfig.json"),
            r#"{
  "include": ["src/**/*"]
}"#,
        )
        .unwrap();

        let files =
            collect_ambient_declaration_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

        assert_eq!(files.len(), 2, "{files:?}");
        assert!(
            files
                .iter()
                .any(|path| path.ends_with("src/@types/globals.d.ts"))
        );
        assert!(files.iter().any(|path| path.ends_with("src/env.d.ts")));
        assert!(!files.iter().any(|path| path.ends_with("src/App.vue")));
        assert!(!files.iter().any(|path| path.ends_with("src/main.ts")));
        assert!(!files.iter().any(|path| path.ends_with("outside.d.ts")));

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn ambient_declaration_collection_skips_module_shim_dts() {
        let case_dir = unique_case_dir("tsconfig-module-shim-dts");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(case_dir.join("src")).unwrap();
        // Module-shim file: its `declare module "vue"` block would shadow the
        // real `vue` package if force-loaded as a program root.
        fs::write(
            case_dir.join("src/shims.d.ts"),
            "declare module \"*.css\";\ndeclare module \"vue\" {\n  export interface GlobalComponents {}\n}\n",
        )
        .unwrap();
        // Genuine ambient-global file: must still be collected.
        fs::write(
            case_dir.join("src/globals.d.ts"),
            "export {};\ndeclare global { type GlobalTabType = 'a' | 'b'; }\n",
        )
        .unwrap();
        // Namespace-style `declare module Foo` is a plain global, not a shim.
        fs::write(
            case_dir.join("src/namespace.d.ts"),
            "declare module Foo { const bar: string; }\n",
        )
        .unwrap();
        fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
        fs::write(
            case_dir.join("tsconfig.json"),
            r#"{
  "include": ["src/**/*"]
}"#,
        )
        .unwrap();

        let files =
            collect_ambient_declaration_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

        assert!(
            files.iter().any(|path| path.ends_with("src/globals.d.ts")),
            "declare-global file should be collected: {files:?}"
        );
        assert!(
            files
                .iter()
                .any(|path| path.ends_with("src/namespace.d.ts")),
            "namespace-style declaration should be collected: {files:?}"
        );
        assert!(
            !files.iter().any(|path| path.ends_with("src/shims.d.ts")),
            "module-shim declaration file should be skipped: {files:?}"
        );

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn default_collection_uses_files_entries() {
        let case_dir = unique_case_dir("tsconfig-files");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(case_dir.join("src")).unwrap();
        fs::write(case_dir.join("src/entry.ts"), "export const ok = true").unwrap();
        fs::write(case_dir.join("src/extra.ts"), "export const extra = true").unwrap();
        fs::write(
            case_dir.join("tsconfig.json"),
            r#"{
  "files": ["src/entry.ts"]
}"#,
        )
        .unwrap();

        let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

        assert_eq!(files, vec![case_dir.join("src/entry.ts")]);

        let _ = fs::remove_dir_all(&case_dir);
    }
}
