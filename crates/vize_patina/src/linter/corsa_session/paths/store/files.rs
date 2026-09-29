use super::walk::{glob_match, is_declaration_file, parse_jsonc, slash_path, walk_glob};
use super::{Path, PathBuf, String, ToCompactString, Value};

pub(super) fn ambient_declaration_files(tsconfig: &Path) -> Vec<String> {
    let Some(base) = tsconfig.parent() else {
        return Vec::new();
    };
    let Ok(content) = std::fs::read_to_string(tsconfig) else {
        return Vec::new();
    };
    let Some(value) = parse_jsonc(&content) else {
        return Vec::new();
    };
    let excludes = exclude_patterns(&value);
    let mut matched = Vec::new();
    if let Some(files) = value.get("files").and_then(Value::as_array) {
        for entry in files.iter().filter_map(Value::as_str) {
            let path = resolve_config_path(base, entry);
            if path.is_file() {
                matched.push(path);
            }
        }
    }
    if let Some(include) = value.get("include").and_then(Value::as_array) {
        for entry in include.iter().filter_map(Value::as_str) {
            collect_include_entry(base, entry, &mut matched);
        }
    }
    if value.get("files").is_none() && value.get("include").is_none() {
        walk_glob(base, &["**", "*.d.ts"], &mut matched);
    }

    let mut declarations = Vec::new();
    for path in matched {
        if !is_declaration_file(&path) {
            continue;
        }
        let relative = path.strip_prefix(base).unwrap_or(path.as_path());
        let relative = slash_path(relative);
        if is_excluded(relative.as_str(), &excludes) {
            continue;
        }
        let absolute = std::fs::canonicalize(&path).unwrap_or(path);
        declarations.push(slash_path(&absolute));
    }
    declarations.sort();
    declarations.dedup();
    declarations
}

fn exclude_patterns(value: &Value) -> Vec<String> {
    match value.get("exclude") {
        Some(Value::Array(entries)) => entries
            .iter()
            .filter_map(Value::as_str)
            .map(ToCompactString::to_compact_string)
            .collect(),
        Some(_) => Vec::new(),
        None => ["node_modules", "bower_components", "jspm_packages"]
            .into_iter()
            .map(ToCompactString::to_compact_string)
            .collect(),
    }
}

fn is_excluded(relative: &str, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }
    let mut prefix = relative;
    loop {
        if patterns
            .iter()
            .any(|pattern| glob_match(pattern.as_str(), prefix))
        {
            return true;
        }
        let Some((parent, _)) = prefix.rsplit_once('/') else {
            return false;
        };
        prefix = parent;
    }
}

fn collect_include_entry(base: &Path, pattern: &str, out: &mut Vec<PathBuf>) {
    if pattern.contains('*') || pattern.contains('?') {
        collect_glob(base, pattern, out);
        return;
    }
    let path = resolve_config_path(base, pattern);
    if path.is_file() {
        out.push(path);
    } else if path.is_dir() {
        walk_glob(&path, &["**", "*.d.ts"], out);
    }
}

fn resolve_config_path(base: &Path, pattern: &str) -> PathBuf {
    let pattern = pattern.trim_start_matches("./");
    let path = Path::new(pattern);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(pattern)
    }
}

fn collect_glob(base: &Path, pattern: &str, out: &mut Vec<PathBuf>) {
    let pattern = pattern.trim_start_matches("./");
    let absolute = Path::new(pattern).is_absolute();
    let mut root = if absolute {
        PathBuf::from("/")
    } else {
        base.to_path_buf()
    };
    let raw_parts: Vec<&str> = pattern.split('/').filter(|part| !part.is_empty()).collect();
    let mut index = 0usize;
    if absolute {
        while raw_parts
            .get(index)
            .is_some_and(|part| !part.contains('*') && !part.contains('?'))
        {
            let Some(part) = raw_parts.get(index) else {
                break;
            };
            root.push(part);
            index += 1;
        }
    }
    let Some(parts) = raw_parts.get(index..) else {
        return;
    };
    if parts.is_empty() {
        if root.is_file() {
            out.push(root);
        }
        return;
    }
    walk_glob(&root, parts, out);
}
