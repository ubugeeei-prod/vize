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
    normalized: std::string::String,
}

impl GlobSpec {
    fn new(base_dir: &Path, value: &str) -> Option<Self> {
        let (base_dir, normalized) = normalize_tsconfig_glob_base(base_dir, value);
        Pattern::new(&normalized).ok().map(|pattern| Self {
            base_dir,
            pattern,
            normalized,
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
    collect_default_check_files_inner(project_root, tsconfig_path, false)
}

fn collect_default_check_files_inner(
    project_root: &Path,
    tsconfig_path: Option<&Path>,
    include_hidden_tsconfig_roots: bool,
) -> Vec<PathBuf> {
    let Some(tsconfig_path) = tsconfig_path else {
        return collect_supported_files(project_root, &[], &[]);
    };

    let mut files = Vec::new();
    let mut seen = FxHashSet::default();
    for tsconfig_path in collect_tsconfig_project_paths(tsconfig_path) {
        collect_default_check_files_for_tsconfig(
            project_root,
            &tsconfig_path,
            include_hidden_tsconfig_roots,
            &mut files,
            &mut seen,
        );
    }

    files.sort();
    files
}

fn collect_default_check_files_for_tsconfig(
    project_root: &Path,
    tsconfig_path: &Path,
    include_hidden_tsconfig_roots: bool,
    files: &mut Vec<PathBuf>,
    seen: &mut FxHashSet<PathBuf>,
) {
    let spec = load_tsconfig_inputs(tsconfig_path).unwrap_or_default();

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
        if include_hidden_tsconfig_roots {
            for root in explicit_hidden_include_roots(project_root, &includes) {
                for path in collect_supported_files_with_options(
                    &root,
                    &includes,
                    &excludes,
                    FileCollectionOptions {
                        include_hidden: true,
                    },
                ) {
                    if seen.insert(path.clone()) {
                        files.push(path);
                    }
                }
            }
        }
    }
}

pub(crate) fn resolve_tsconfig_for_files(
    tsconfig_path: Option<&Path>,
    files: &[PathBuf],
) -> Option<PathBuf> {
    let tsconfig_path = tsconfig_path?;
    let projects = collect_tsconfig_project_paths(tsconfig_path);
    let root_project = projects
        .first()
        .cloned()
        .unwrap_or_else(|| normalize_input_path(tsconfig_path));
    let files = files
        .iter()
        .filter(|path| is_supported_check_file(path))
        .map(|path| normalize_input_path(path))
        .collect::<Vec<_>>();
    if files.is_empty() {
        return Some(root_project);
    }

    if let Some(owner) = projects
        .iter()
        .find(|project| files.iter().all(|file| tsconfig_owns_file(project, file)))
    {
        return Some(owner.clone());
    }

    let mut shared_owner = None::<PathBuf>;
    for file in &files {
        let Some(owner) = projects
            .iter()
            .find(|project| tsconfig_owns_file(project, file))
        else {
            return Some(root_project);
        };
        match &shared_owner {
            Some(shared) if shared != owner => return Some(root_project),
            Some(_) => {}
            None => shared_owner = Some(owner.clone()),
        }
    }

    shared_owner.or(Some(root_project))
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
/// Project shims such as `declare module "~icons/foo"` and Nuxt's generated
/// `.nuxt/nuxt.d.ts` are part of that program even though they are not imported
/// by the checked file. Only unsafe bare Vue package shims without top-level
/// import/export are excluded, because those replace the real package instead
/// of augmenting it.
pub(crate) fn collect_ambient_declaration_files(
    project_root: &Path,
    tsconfig_path: Option<&Path>,
) -> Vec<PathBuf> {
    let project_root = normalize_input_path(project_root);
    let mut files = collect_default_check_files_inner(&project_root, tsconfig_path, true);
    let mut seen = files.iter().cloned().collect::<FxHashSet<_>>();
    let mut index = 0;
    while index < files.len() {
        let path = files[index].clone();
        index += 1;
        if !is_declaration_file(&path) {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        for referenced in reference_path_declaration_files(&path, &content, &project_root) {
            if seen.insert(referenced.clone()) {
                files.push(referenced);
            }
        }
    }

    files
        .into_iter()
        .filter(|path| is_declaration_file(path))
        .filter(|path| match fs::read_to_string(path) {
            Ok(content) => {
                !is_reference_manifest_declaration(&content)
                    && !declares_shadowing_ambient_module(&content)
            }
            Err(_) => false,
        })
        .collect()
}

fn is_declaration_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
}

/// Returns `true` when a declaration file would replace real Vue package types
/// if loaded as a program root. Project shims such as `declare module "*.css"`
/// and `declare module "~icons/foo"` must still be loaded for explicit checks,
/// while bare ambient `declare module "vue"` files without top-level imports or
/// exports shadow the real package.
fn declares_shadowing_ambient_module(content: &str) -> bool {
    if has_top_level_import_or_export(content) {
        return false;
    }

    ambient_module_specifiers(content)
        .iter()
        .any(|specifier| is_shadowed_vue_package_specifier(specifier))
}

fn ambient_module_specifiers(content: &str) -> Vec<std::string::String> {
    const NEEDLE: &str = "declare module";
    let mut specifiers = Vec::new();
    for (index, _) in content.match_indices(NEEDLE) {
        let preceded_by_boundary = content[..index]
            .chars()
            .next_back()
            .is_none_or(|ch| !ch.is_alphanumeric() && ch != '_' && ch != '$');
        if !preceded_by_boundary {
            continue;
        }
        let mut chars = content[index + NEEDLE.len()..].chars();
        let Some(quote) = chars.find(|ch| !ch.is_whitespace()) else {
            continue;
        };
        if quote != '"' && quote != '\'' {
            continue;
        }
        let mut specifier = std::string::String::new();
        let mut escaped = false;
        for ch in chars {
            if escaped {
                specifier.push(ch);
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote {
                specifiers.push(specifier);
                break;
            } else {
                specifier.push(ch);
            }
        }
    }
    specifiers
}

fn reference_path_declaration_files(
    path: &Path,
    content: &str,
    project_root: &Path,
) -> Vec<PathBuf> {
    let Some(base_dir) = path.parent() else {
        return Vec::new();
    };
    content
        .lines()
        .filter_map(reference_path_attribute)
        .filter_map(|reference| {
            let resolved = normalize_input_path(&base_dir.join(reference));
            (resolved.starts_with(project_root)
                && !path_has_component(&resolved, NODE_MODULES_DIR)
                && is_declaration_file(&resolved)
                && resolved.is_file())
            .then_some(resolved)
        })
        .collect()
}

fn reference_path_attribute(line: &str) -> Option<&str> {
    let line = line.trim_start();
    if !line.starts_with("///") || !line.contains("<reference") {
        return None;
    }
    attribute_value(line, "path")
}

fn attribute_value<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("{name}=");
    let start = line.find(&needle)? + needle.len();
    let quote = line[start..].chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let value_start = start + quote.len_utf8();
    let value_end = line[value_start..].find(quote)? + value_start;
    line.get(value_start..value_end)
}

fn is_reference_manifest_declaration(content: &str) -> bool {
    let mut has_reference = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("///") && trimmed.contains("<reference") {
            has_reference = true;
            continue;
        }
        if matches!(trimmed, "export {}" | "export {};") {
            continue;
        }
        return false;
    }
    has_reference
}

fn has_top_level_import_or_export(content: &str) -> bool {
    content.lines().any(|line| {
        line.starts_with("import ")
            || line.starts_with("import{")
            || line.starts_with("export ")
            || line.starts_with("export{")
            || line.starts_with("export {}")
    })
}

fn is_shadowed_vue_package_specifier(specifier: &str) -> bool {
    matches!(
        specifier,
        "vue" | "@vue/runtime-core" | "@vue/runtime-dom" | "vue-router"
    )
}

pub(crate) fn load_tsconfig_declaration_options(
    tsconfig_path: &Path,
) -> TsconfigDeclarationOptions {
    let mut seen = FxHashSet::default();
    load_tsconfig_declaration_options_inner(tsconfig_path, &mut seen).unwrap_or_default()
}

#[derive(Debug, Clone, Copy, Default)]
struct FileCollectionOptions {
    include_hidden: bool,
}

fn collect_supported_files(
    root: &Path,
    includes: &[GlobSpec],
    excludes: &[GlobSpec],
) -> Vec<PathBuf> {
    collect_supported_files_with_options(root, includes, excludes, FileCollectionOptions::default())
}

fn collect_supported_files_with_options(
    root: &Path,
    includes: &[GlobSpec],
    excludes: &[GlobSpec],
    options: FileCollectionOptions,
) -> Vec<PathBuf> {
    // Keep the tsconfig scan ignore-aware and canonicalize only the root. The
    // matched files are sorted after collection, so the parallel walk can avoid
    // expensive per-entry canonicalization without making CLI output unstable.
    let skip_generated = should_skip_generated_for_root(root);
    let normalized_root = normalize_input_path(root);
    let walker = WalkBuilder::new(root)
        .standard_filters(true)
        .hidden(!options.include_hidden)
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

fn explicit_hidden_include_roots(project_root: &Path, includes: &[GlobSpec]) -> Vec<PathBuf> {
    let normalized_project_root = normalize_input_path(project_root);
    let mut roots = Vec::new();
    let mut seen = FxHashSet::default();

    for include in includes {
        if path_has_hidden_component_under_root(&include.base_dir, &normalized_project_root) {
            push_hidden_include_root(&mut roots, &mut seen, &include.base_dir);
        }
        if let Some(root) = hidden_pattern_root(&include.base_dir, &include.normalized) {
            push_hidden_include_root(&mut roots, &mut seen, &root);
        }
    }

    roots
}

fn push_hidden_include_root(roots: &mut Vec<PathBuf>, seen: &mut FxHashSet<PathBuf>, root: &Path) {
    let root = normalize_input_path(root);
    if root.is_dir() && seen.insert(root.clone()) {
        roots.push(root);
    }
}

fn path_has_hidden_component_under_root(path: &Path, root: &Path) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(is_hidden_path_segment)
    })
}

fn path_has_component(path: &Path, component_name: &str) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|name| name == component_name)
    })
}

fn hidden_pattern_root(base_dir: &Path, pattern: &str) -> Option<PathBuf> {
    let mut root = base_dir.to_path_buf();
    for segment in pattern.split('/') {
        if segment.is_empty() {
            continue;
        }
        if segment.contains(['*', '?', '[']) {
            break;
        }
        root.push(segment);
        if is_hidden_path_segment(segment) {
            return Some(root);
        }
    }
    None
}

fn is_hidden_path_segment(segment: &str) -> bool {
    segment.starts_with('.') && segment != "." && segment != ".."
}

fn matches_tsconfig_patterns(path: &Path, includes: &[GlobSpec], excludes: &[GlobSpec]) -> bool {
    if !includes.is_empty() && !includes.iter().any(|glob| glob.matches(path)) {
        return false;
    }

    !excludes.iter().any(|glob| glob.matches(path))
}

fn tsconfig_owns_file(tsconfig_path: &Path, file: &Path) -> bool {
    let Some(spec) = load_tsconfig_inputs(tsconfig_path) else {
        return false;
    };
    let file = normalize_input_path(file);
    if spec
        .files
        .iter()
        .any(|entry| normalize_input_path(&entry.resolve()) == file)
    {
        return true;
    }

    let default_base_dir = tsconfig_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    let includes = if !spec.has_includes && !spec.has_files {
        GlobSpec::new(&default_base_dir, "**/*")
            .into_iter()
            .collect::<Vec<_>>()
    } else {
        spec.includes
    };
    if includes.is_empty() || !is_supported_check_file(&file) {
        return false;
    }
    let excludes = if !spec.has_excludes {
        default_exclude_specs(&default_base_dir)
    } else {
        spec.excludes
    };

    matches_tsconfig_patterns(&file, &includes, &excludes)
}

fn collect_tsconfig_project_paths(tsconfig_path: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen = FxHashSet::default();
    collect_tsconfig_project_paths_inner(tsconfig_path, &mut seen, &mut paths);
    paths
}

fn collect_tsconfig_project_paths_inner(
    tsconfig_path: &Path,
    seen: &mut FxHashSet<PathBuf>,
    paths: &mut Vec<PathBuf>,
) {
    let resolved = normalize_input_path(tsconfig_path);
    if !seen.insert(resolved.clone()) {
        return;
    }
    paths.push(resolved.clone());

    let Ok(content) = tracked_read_to_string(&resolved) else {
        return;
    };
    let value = parse_jsonc_value(&content).unwrap_or(Value::Null);
    for reference in read_reference_entries(&value) {
        let Some(reference_path) = resolve_referenced_tsconfig(&resolved, &reference) else {
            continue;
        };
        collect_tsconfig_project_paths_inner(&reference_path, seen, paths);
    }
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

pub(super) fn resolve_extended_tsconfig(tsconfig_path: &Path, extends: &str) -> Option<PathBuf> {
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

fn resolve_referenced_tsconfig(tsconfig_path: &Path, reference: &str) -> Option<PathBuf> {
    let base_dir = tsconfig_path.parent().unwrap_or(Path::new("."));
    let reference_path = Path::new(reference);
    let base = if reference_path.is_absolute() {
        reference_path.to_path_buf()
    } else {
        base_dir.join(reference_path)
    };
    let mut candidates = Vec::new();
    push_tsconfig_candidates(&mut candidates, base);
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

pub(super) fn read_extends_entries(value: &Value) -> Vec<std::string::String> {
    match value.get("extends") {
        Some(Value::String(extends)) => vec![extends.clone()],
        Some(Value::Array(extends)) => extends
            .iter()
            .filter_map(|item| item.as_str().map(std::string::String::from))
            .collect(),
        _ => Vec::new(),
    }
}

fn read_reference_entries(value: &Value) -> Vec<std::string::String> {
    value
        .get("references")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| item.get("path").and_then(Value::as_str))
        .map(std::string::String::from)
        .collect()
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

fn normalize_tsconfig_glob_base(base_dir: &Path, value: &str) -> (PathBuf, std::string::String) {
    let mut base_dir = base_dir.to_path_buf();
    let mut normalized = normalize_tsconfig_glob(value);

    loop {
        if let Some(rest) = normalized.strip_prefix("./") {
            normalized = rest.to_owned();
        } else if let Some(rest) = normalized.strip_prefix("../") {
            if let Some(parent) = base_dir.parent() {
                base_dir = parent.to_path_buf();
            }
            normalized = rest.to_owned();
        } else {
            break;
        }
    }

    if normalized.is_empty() {
        normalized.push_str("**/*");
    }

    (base_dir, normalized)
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

pub(super) fn parse_jsonc_value(content: &str) -> Result<Value, serde_json::Error> {
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
        load_tsconfig_declaration_options, resolve_extended_tsconfig, resolve_tsconfig_for_files,
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
    fn default_collection_matches_parent_relative_extended_include() {
        let case_dir = unique_case_dir("tsconfig-extends-parent-relative");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(case_dir.join(".nuxt")).unwrap();
        fs::create_dir_all(case_dir.join("src")).unwrap();
        fs::create_dir_all(case_dir.join("dist")).unwrap();
        fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
        fs::write(case_dir.join("src/main.ts"), "export const ok = true").unwrap();
        fs::write(
            case_dir.join("dist/generated.ts"),
            "export const skip = true",
        )
        .unwrap();
        fs::write(case_dir.join(".nuxt/nuxt.d.ts"), "declare const nuxt: true").unwrap();
        fs::write(
            case_dir.join(".nuxt/tsconfig.json"),
            r#"{
  "include": ["./nuxt.d.ts", "../src/**/*", "../dist/**/*.ts"],
  "exclude": ["../dist"]
}"#,
        )
        .unwrap();
        fs::write(
            case_dir.join("tsconfig.json"),
            r#"{
  "extends": "./.nuxt/tsconfig.json"
}"#,
        )
        .unwrap();

        let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

        assert!(files.iter().any(|path| path.ends_with("src/App.vue")));
        assert!(files.iter().any(|path| path.ends_with("src/main.ts")));
        assert!(!files.iter().any(|path| path.ends_with("dist/generated.ts")));

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
    fn ambient_declaration_collection_keeps_project_shims_but_skips_vue_shadows() {
        let case_dir = unique_case_dir("tsconfig-module-shim-dts");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(case_dir.join("src")).unwrap();
        fs::create_dir_all(case_dir.join(".nuxt/types")).unwrap();
        // This file would shadow the real `vue` package if force-loaded as a
        // program root, so it must remain excluded.
        fs::write(
            case_dir.join("src/vue-shadow.d.ts"),
            "declare module \"vue\" {\n  export interface GlobalComponents {}\n}\n",
        )
        .unwrap();
        // Project shims are needed for explicit checks: no source import can
        // discover these declarations otherwise.
        fs::write(
            case_dir.join("src/project-shims.d.ts"),
            "declare module \"*.css\";\ndeclare module \"~icons/foo\";\n",
        )
        .unwrap();
        // Nuxt/Vue package augmentations are safe when the declaration file is
        // an external module.
        fs::write(
            case_dir.join("src/vue-augmentation.d.ts"),
            "import \"vue\";\ndeclare module \"vue\" {\n  export interface GlobalComponents {}\n}\nexport {};\n",
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
        // Hidden tsconfig roots such as `.nuxt` are excluded by the normal
        // default scanner but must still be loaded as ambient roots.
        fs::write(
            case_dir.join(".nuxt/nuxt.d.ts"),
            "/// <reference path=\"types/feature-flags.d.ts\" />\nexport {};\n",
        )
        .unwrap();
        fs::write(
            case_dir.join(".nuxt/types/feature-flags.d.ts"),
            "export {};\ndeclare global { interface ImportMeta { vfFeatures: { enabled: boolean }; } }\n",
        )
        .unwrap();
        fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
        fs::write(
            case_dir.join("tsconfig.json"),
            r#"{
  "include": ["src/**/*", ".nuxt/nuxt.d.ts"]
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
            files
                .iter()
                .any(|path| path.ends_with("src/project-shims.d.ts")),
            "project module shims should be collected: {files:?}"
        );
        assert!(
            files
                .iter()
                .any(|path| path.ends_with("src/vue-augmentation.d.ts")),
            "external-module Vue augmentations should be collected: {files:?}"
        );
        assert!(
            !files.iter().any(|path| path.ends_with(".nuxt/nuxt.d.ts")),
            "reference-only Nuxt declaration manifest should be expanded but not collected: {files:?}"
        );
        assert!(
            files
                .iter()
                .any(|path| path.ends_with(".nuxt/types/feature-flags.d.ts")),
            "hidden Nuxt referenced declaration should be collected: {files:?}"
        );
        assert!(
            !files
                .iter()
                .any(|path| path.ends_with("src/vue-shadow.d.ts")),
            "ambient Vue shadow declaration file should be skipped: {files:?}"
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

    #[test]
    fn default_collection_follows_referenced_tsconfigs() {
        let case_dir = unique_case_dir("tsconfig-references");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(case_dir.join(".generated")).unwrap();
        fs::create_dir_all(case_dir.join("src")).unwrap();
        fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
        fs::write(
            case_dir.join(".generated/types.d.ts"),
            "declare const X: true",
        )
        .unwrap();
        fs::write(
            case_dir.join("tsconfig.json"),
            r#"{
  "files": [],
  "references": [{ "path": "./.generated/tsconfig.app.json" }]
}"#,
        )
        .unwrap();
        fs::write(
            case_dir.join(".generated/tsconfig.app.json"),
            r#"{
  "include": ["./types.d.ts", "../src/**/*.vue"]
}"#,
        )
        .unwrap();

        let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

        assert!(files.iter().any(|path| path.ends_with("src/App.vue")));
        assert!(
            !files
                .iter()
                .any(|path| path.ends_with(".generated/types.d.ts")),
            "normal default scan keeps hidden generated roots ignored: {files:?}"
        );

        let ambient =
            collect_ambient_declaration_files(&case_dir, Some(&case_dir.join("tsconfig.json")));
        assert!(
            ambient
                .iter()
                .any(|path| path.ends_with(".generated/types.d.ts")),
            "ambient scan should include hidden referenced declaration roots: {ambient:?}"
        );

        let _ = fs::remove_dir_all(&case_dir);
    }

    #[test]
    fn tsconfig_for_files_uses_referenced_owner() {
        let case_dir = unique_case_dir("tsconfig-reference-owner");
        let _ = fs::remove_dir_all(&case_dir);
        fs::create_dir_all(case_dir.join(".generated")).unwrap();
        fs::create_dir_all(case_dir.join("src")).unwrap();
        let app = case_dir.join("src/App.vue");
        fs::write(&app, "<template />").unwrap();
        fs::write(
            case_dir.join("tsconfig.json"),
            r#"{
  "files": [],
  "references": [{ "path": "./.generated/tsconfig.app.json" }]
}"#,
        )
        .unwrap();
        fs::write(
            case_dir.join(".generated/tsconfig.app.json"),
            r#"{
  "include": ["../src/**/*.vue"]
}"#,
        )
        .unwrap();

        let owner = resolve_tsconfig_for_files(Some(&case_dir.join("tsconfig.json")), &[app]);

        assert_eq!(owner, Some(case_dir.join(".generated/tsconfig.app.json")));

        let _ = fs::remove_dir_all(&case_dir);
    }
}
