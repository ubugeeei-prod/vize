use super::{InheritedPaths, Path, PathBuf};

pub(super) fn inherited_paths(
    config_path: &Path,
    stack: &mut Vec<PathBuf>,
) -> Option<InheritedPaths> {
    let normalized = normalize_path_lexically(config_path);
    if stack.iter().any(|seen| seen == &normalized) {
        return None;
    }
    let value = read_jsonc(config_path)?;
    stack.push(normalized);
    let mut inherited = InheritedPaths {
        paths_dir: None,
        entries: Vec::new(),
        base_url_dir: None,
        base_url: None,
    };
    if let Some(extends) = value.get("extends") {
        for spec in extend_specs(extends) {
            let Some(resolved) = resolve_extended_tsconfig_path(config_path, spec) else {
                continue;
            };
            let Some(parent) = inherited_paths(&resolved, stack) else {
                continue;
            };
            if parent.paths_dir.is_some() {
                inherited.paths_dir = parent.paths_dir;
                inherited.entries = parent.entries;
            }
            if parent.base_url.is_some() {
                inherited.base_url_dir = parent.base_url_dir;
                inherited.base_url = parent.base_url;
            }
        }
    }
    let config_dir = config_path.parent().unwrap_or(Path::new("."));
    if let Some(compiler_options) = value.get("compilerOptions") {
        if let Some(entries) = own_path_entries(compiler_options) {
            inherited.paths_dir = Some(config_dir.to_path_buf());
            inherited.entries = entries;
        }
        if let Some(base_url) = compiler_options.get("baseUrl").and_then(|v| v.as_str()) {
            inherited.base_url_dir = Some(config_dir.to_path_buf());
            inherited.base_url = Some(base_url.to_owned());
        }
    }
    stack.pop();
    Some(inherited)
}

#[expect(
    clippy::disallowed_types,
    reason = "serde_json object keys and string values are std String"
)]
fn own_path_entries(
    compiler_options: &serde_json::Value,
) -> Option<Vec<(std::string::String, std::string::String)>> {
    let paths = compiler_options.get("paths")?.as_object()?;
    let mut entries = Vec::new();
    for (pattern, targets) in paths {
        for target in targets.as_array().into_iter().flatten() {
            if let Some(target) = target.as_str() {
                entries.push((pattern.clone(), target.to_owned()));
            }
        }
    }
    Some(entries)
}

fn extend_specs(extends: &serde_json::Value) -> Vec<&str> {
    match extends {
        serde_json::Value::String(spec) => vec![spec.as_str()],
        serde_json::Value::Array(items) => items.iter().filter_map(|item| item.as_str()).collect(),
        _ => Vec::new(),
    }
}

/// Resolve one `extends` entry the way TypeScript does: relative and absolute
/// paths (a directory means `tsconfig.json` inside it), or a bare package
/// specifier walked through ancestor `node_modules`.
fn resolve_extended_tsconfig_path(tsconfig_path: &Path, extends: &str) -> Option<PathBuf> {
    let base_dir = tsconfig_path.parent().unwrap_or(Path::new("."));
    let extends_path = Path::new(extends);
    if !(extends_path.is_absolute()
        || extends.starts_with("./")
        || extends.starts_with("../")
        || extends == "."
        || extends == "..")
    {
        return resolve_package_tsconfig_path(base_dir, extends);
    }
    let base = if extends_path.is_absolute() {
        extends_path.to_path_buf()
    } else {
        base_dir.join(extends_path)
    };
    tsconfig_path_candidates(base)
        .into_iter()
        .map(|candidate| normalize_path_lexically(&candidate))
        .find(|candidate| candidate.is_file())
}

fn resolve_package_tsconfig_path(base_dir: &Path, extends: &str) -> Option<PathBuf> {
    let (package, subpath) = split_package_specifier(extends)?;
    let mut current = Some(base_dir);
    while let Some(dir) = current {
        let package_root = dir.join("node_modules").join(package);
        let from_package_json = if subpath.is_none() {
            read_jsonc(&package_root.join("package.json")).and_then(|package_json| {
                package_json
                    .get("tsconfig")
                    .and_then(|target| target.as_str())
                    .map(|target| package_root.join(target))
            })
        } else {
            None
        };
        let base = subpath.map_or_else(
            || package_root.join("tsconfig.json"),
            |subpath| package_root.join(subpath),
        );
        if let Some(found) = from_package_json
            .into_iter()
            .flat_map(tsconfig_path_candidates)
            .chain(tsconfig_path_candidates(base))
            .map(|candidate| normalize_path_lexically(&candidate))
            .find(|candidate| candidate.is_file())
        {
            return Some(std::fs::canonicalize(&found).unwrap_or(found));
        }
        current = dir.parent();
    }
    None
}

fn split_package_specifier(extends: &str) -> Option<(&str, Option<&str>)> {
    if let Some(scoped) = extends.strip_prefix('@') {
        let (scope, rest) = scoped.split_once('/')?;
        if scope.is_empty() {
            return None;
        }
        return match rest.split_once('/') {
            Some((name, subpath)) if !name.is_empty() && !subpath.is_empty() => {
                let package = extends.strip_suffix(subpath)?.strip_suffix('/')?;
                Some((package, Some(subpath)))
            }
            Some((name, subpath)) if !name.is_empty() && subpath.is_empty() => {
                Some((extends.strip_suffix('/').unwrap_or(extends), None))
            }
            None if !rest.is_empty() => Some((extends, None)),
            _ => None,
        };
    }
    match extends.split_once('/') {
        Some((name, subpath)) if !name.is_empty() && !subpath.is_empty() => {
            Some((name, Some(subpath)))
        }
        Some((name, subpath)) if !name.is_empty() && subpath.is_empty() => Some((name, None)),
        None if !extends.is_empty() => Some((extends, None)),
        _ => None,
    }
}

fn tsconfig_path_candidates(base: PathBuf) -> Vec<PathBuf> {
    if base.extension().is_some() {
        return vec![base];
    }
    vec![
        base.clone(),
        base.with_extension("json"),
        base.join("tsconfig.json"),
    ]
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
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

/// The project configs a solution-style shell references, in declaration
/// order; a `path` may name a config file or a directory.
pub(super) fn referenced_configs(config_path: &Path) -> Vec<PathBuf> {
    let Some(value) = read_jsonc(config_path) else {
        return Vec::new();
    };
    let Some(references) = value.get("references").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let base = config_path.parent().unwrap_or(Path::new("."));
    references
        .iter()
        .filter_map(|reference| reference.get("path").and_then(|p| p.as_str()))
        .filter_map(|path| {
            let joined = base.join(path);
            if joined.is_file() {
                return Some(joined);
            }
            let as_directory = joined.join("tsconfig.json");
            as_directory.is_file().then_some(as_directory)
        })
        .collect()
}

fn read_jsonc(path: &Path) -> Option<serde_json::Value> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content)
        .ok()
        .or_else(|| serde_json::from_str(&strip_jsonc_sugar(&content)).ok())
}

/// Reduce the JSONC that TypeScript accepts to the JSON `serde_json` parses:
/// comments and trailing commas, both of which `tsc` allows anywhere. String
/// state is tracked throughout, because every `paths` pattern contains `/*`
/// (`"@/*"`) and a stripper that ignores it destroys the value we came for.
#[expect(
    clippy::disallowed_types,
    reason = "serde_json object keys and string values are std String"
)]
pub(super) fn strip_jsonc_sugar(source: &str) -> std::string::String {
    let mut out = std::string::String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                out.push(c);
            }
            '/' if chars.peek() == Some(&'/') => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut last = ' ';
                for c in chars.by_ref() {
                    if last == '*' && c == '/' {
                        break;
                    }
                    last = c;
                }
            }
            // A closing brace or bracket retroactively makes any comma that
            // precedes it (across whitespace and stripped comments) trailing.
            '}' | ']' => {
                while out.ends_with(char::is_whitespace) {
                    out.pop();
                }
                if out.ends_with(',') {
                    out.pop();
                }
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}
