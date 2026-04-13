//! `tsconfig.json`-driven default input collection for `vize check`.
//!
//! When users run `vize check` without explicit paths, we should follow the
//! project's configured `files` / `include` / `exclude` fields instead of
//! recursively scanning every TypeScript file under the working directory.

#![allow(clippy::disallowed_types)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use glob::{MatchOptions, Pattern};
use ignore::WalkBuilder;
use serde_json::Value;
use vize_carton::FxHashSet;

#[derive(Debug, Clone, Default)]
struct TsconfigInputSpec {
    files: Vec<RelativePathSpec>,
    includes: Vec<GlobSpec>,
    excludes: Vec<GlobSpec>,
    has_files: bool,
    has_includes: bool,
    has_excludes: bool,
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

fn collect_supported_files(
    root: &Path,
    includes: &[GlobSpec],
    excludes: &[GlobSpec],
) -> Vec<PathBuf> {
    let skip_generated = should_skip_generated_for_root(root);
    let walker = WalkBuilder::new(root)
        .standard_filters(true)
        .hidden(true)
        .build_parallel();

    let collected = std::sync::Mutex::new(Vec::<PathBuf>::new());
    walker.run(|| {
        let collected = &collected;
        Box::new(move |entry| {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file()
                    && is_supported_check_file(path)
                    && (!skip_generated || !is_generated_path(path))
                    && matches_tsconfig_patterns(path, includes, excludes)
                {
                    if let Ok(mut collected) = collected.lock() {
                        collected.push(normalize_input_path(path));
                    }
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

    let content = fs::read_to_string(&resolved)?;
    let value = parse_jsonc_value(&content).unwrap_or(Value::Null);
    let dir = resolved.parent().unwrap_or(Path::new("."));

    let mut merged = value
        .get("extends")
        .and_then(Value::as_str)
        .and_then(|extends| resolve_extended_tsconfig(&resolved, extends))
        .map(|extends_path| load_tsconfig_inputs_inner(&extends_path, seen))
        .transpose()?
        .unwrap_or_default();

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
        push_tsconfig_candidates(&mut candidates, base_dir.join("node_modules").join(extends));
    }

    candidates.into_iter().find(|candidate| candidate.exists())
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

fn should_skip_generated_for_root(root: &Path) -> bool {
    !root
        .components()
        .any(|component| component.as_os_str().to_str() == Some("__agent_only"))
}

fn is_generated_path(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|name| matches!(name, "__agent_only" | "target"))
    })
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
    use super::collect_default_check_files;
    use std::fs;
    use std::path::{Path, PathBuf};
    use vize_carton::cstr;

    fn unique_case_dir(name: &str) -> PathBuf {
        static NEXT_CASE_ID: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("__agent_only")
            .join("tests")
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
        assert!(!files
            .iter()
            .any(|path| path.ends_with("src/generated/skip.ts")));

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
