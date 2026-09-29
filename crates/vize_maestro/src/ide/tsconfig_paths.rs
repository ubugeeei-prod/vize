//! Shared `tsconfig.json` `paths` reading for editor features (#3915, #3917).
//!
//! Anchoring matches the session's rule: the nearest `tsconfig.json` governs;
//! when it is a solution-style shell that declares no `paths` of its own, the
//! first referenced project config that does wins (the create-vue app/node
//! split has exactly one). `extends` merges the way TypeScript merges it: a
//! later config replaces `paths` and `baseUrl` wholesale, and relative `paths`
//! targets resolve against the effective `baseUrl` when any config in the
//! chain declares one, otherwise against the directory of the config that
//! declared the winning `paths` map. Comment stripping is string-aware — every
//! `paths` pattern contains `/*` (`"@/*"`), so a stripper that ignores string
//! state destroys exactly the value these features need.
#![expect(
    clippy::disallowed_types,
    reason = "tower-lsp lsp_types take std String/HashMap values, built with to_string/format!"
)]

use std::path::{Path, PathBuf};

/// The effective `paths` map for `source_path`: the directory targets resolve
/// against (`compilerOptions.baseUrl` when set, otherwise the declaring
/// config's own directory), and the (pattern, target) pairs spelled as written.
pub(crate) struct ProjectPaths {
    pub(crate) anchor: PathBuf,
    pub(crate) entries: Vec<(std::string::String, std::string::String)>,
}

pub(crate) fn project_paths(source_path: &Path) -> Option<ProjectPaths> {
    let anchor = source_path
        .ancestors()
        .skip(1)
        .find(|dir| dir.join("tsconfig.json").is_file())?;
    let shell = anchor.join("tsconfig.json");
    let mut stack = Vec::new();
    if let Some(paths) = paths_of(&shell, &mut stack) {
        return Some(paths);
    }
    referenced_configs(&shell)
        .into_iter()
        .find_map(|referenced| paths_of(&referenced, &mut Vec::new()))
}

/// `paths` and `baseUrl` after an `extends` merge. Each anchor directory is
/// the config whose declaration survived, not the file that was asked for.
struct InheritedPaths {
    paths_dir: Option<PathBuf>,
    entries: Vec<(std::string::String, std::string::String)>,
    base_url_dir: Option<PathBuf>,
    base_url: Option<std::string::String>,
}

fn paths_of(config_path: &Path, stack: &mut Vec<PathBuf>) -> Option<ProjectPaths> {
    let inherited = inherited_paths(config_path, stack)?;
    if inherited.entries.is_empty() {
        return None;
    }
    // A relative target is resolved against the effective `baseUrl` when one
    // survived the merge, and against the winning `paths` map's directory
    // otherwise. Targets stay spelled as written; callers join them to `anchor`.
    let anchor = match (&inherited.base_url, &inherited.base_url_dir) {
        (Some(base_url), Some(dir)) => dir.join(base_url),
        _ => inherited.paths_dir?,
    };
    Some(ProjectPaths {
        anchor,
        entries: inherited.entries,
    })
}

fn inherited_paths(config_path: &Path, stack: &mut Vec<PathBuf>) -> Option<InheritedPaths> {
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
fn referenced_configs(config_path: &Path) -> Vec<PathBuf> {
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
fn strip_jsonc_sugar(source: &str) -> std::string::String {
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

#[cfg(test)]
mod tests {
    use std::fs;

    fn temp_dir() -> tempfile::TempDir {
        let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("vize-tests");
        fs::create_dir_all(&base).unwrap();
        tempfile::tempdir_in(base).unwrap()
    }

    #[test]
    fn base_url_anchors_path_targets() {
        let dir = temp_dir();
        let root = dir.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("tsconfig.json"),
            r#"{ "compilerOptions": { "baseUrl": "./src", "paths": { "@/*": ["*"] } } }"#,
        )
        .unwrap();
        let paths = super::project_paths(&root.join("src/App.vue")).unwrap();
        // `Path` equality normalizes the `.` away, so `<root>/./src` matches.
        assert_eq!(paths.anchor, root.join("src"));
        assert_eq!(
            paths.entries,
            vec![("@/*".to_string(), "*".to_string())],
            "targets stay spelled as written"
        );
    }

    #[test]
    fn config_directory_anchors_path_targets_without_base_url() {
        let dir = temp_dir();
        let root = dir.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("tsconfig.json"),
            r#"{ "compilerOptions": { "paths": { "@/*": ["./src/*"] } } }"#,
        )
        .unwrap();
        let paths = super::project_paths(&root.join("src/App.vue")).unwrap();
        assert_eq!(paths.anchor, root);
    }

    #[test]
    fn trailing_commas_and_comments_still_yield_paths() {
        let dir = temp_dir();
        let root = dir.path();
        fs::create_dir_all(root.join("src")).unwrap();
        // Everything `tsc` tolerates at once: comments, and trailing commas in
        // the target array, the `paths` object, and `compilerOptions`.
        fs::write(
            root.join("tsconfig.json"),
            r#"{
  // aliases
  "compilerOptions": {
    "paths": {
      "@/*": ["./src/*",], /* block */
      "~/*": ["./src/*",],
    },
  },
}"#,
        )
        .unwrap();
        let paths = super::project_paths(&root.join("src/App.vue")).unwrap();
        assert_eq!(
            paths.entries,
            vec![
                ("@/*".to_string(), "./src/*".to_string()),
                ("~/*".to_string(), "./src/*".to_string()),
            ]
        );
    }

    #[test]
    fn comments_strip_without_touching_path_patterns() {
        let source = r#"{
  // line comment
  /* block "@/decoy" comment */
  "compilerOptions": { "paths": { "@/*": ["./src/*"] } }
}"#;
        let value: serde_json::Value =
            serde_json::from_str(&super::strip_jsonc_sugar(source)).unwrap();
        assert_eq!(
            value["compilerOptions"]["paths"]["@/*"][0],
            serde_json::json!("./src/*")
        );
    }

    #[test]
    fn extends_loads_paths_anchored_at_the_base_config() {
        let dir = temp_dir();
        let root = dir.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("gen")).unwrap();
        fs::write(
            root.join("gen/tsconfig.json"),
            r##"{ "compilerOptions": { "paths": { "#c/*": ["../src/*"] } } }"##,
        )
        .unwrap();
        fs::write(
            root.join("tsconfig.json"),
            r#"{ "extends": "./gen/tsconfig.json" }"#,
        )
        .unwrap();
        let paths = super::project_paths(&root.join("src/App.vue")).unwrap();
        assert_eq!(paths.anchor, root.join("gen"));
        assert_eq!(
            paths.entries,
            vec![("#c/*".to_string(), "../src/*".to_string())]
        );
    }

    #[test]
    fn child_paths_replace_extended_paths() {
        let dir = temp_dir();
        let root = dir.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("base.json"),
            r##"{ "compilerOptions": { "paths": { "#c/*": ["../src/*"] } } }"##,
        )
        .unwrap();
        fs::write(
            root.join("tsconfig.json"),
            r#"{ "extends": "./base.json", "compilerOptions": { "paths": { "@/*": ["./src/*"] } } }"#,
        )
        .unwrap();
        let paths = super::project_paths(&root.join("src/App.vue")).unwrap();
        assert_eq!(paths.anchor, root);
        assert_eq!(
            paths.entries,
            vec![("@/*".to_string(), "./src/*".to_string())]
        );
    }

    #[test]
    fn child_base_url_reanchors_inherited_paths() {
        let dir = temp_dir();
        let root = dir.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("base.json"),
            r#"{ "compilerOptions": { "paths": { "@/*": ["*"] } } }"#,
        )
        .unwrap();
        fs::write(
            root.join("tsconfig.json"),
            r#"{ "extends": "./base.json", "compilerOptions": { "baseUrl": "./src" } }"#,
        )
        .unwrap();
        let paths = super::project_paths(&root.join("src/App.vue")).unwrap();
        assert_eq!(paths.anchor, root.join("src"));
        assert_eq!(paths.entries, vec![("@/*".to_string(), "*".to_string())]);
    }

    #[test]
    fn later_extends_array_entry_replaces_earlier_paths() {
        let dir = temp_dir();
        let root = dir.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("first.json"),
            r#"{ "compilerOptions": { "paths": { "@/*": ["./src/*"] } } }"#,
        )
        .unwrap();
        fs::write(
            root.join("second.json"),
            r##"{ "compilerOptions": { "paths": { "#c/*": ["./src/*"] } } }"##,
        )
        .unwrap();
        fs::write(
            root.join("tsconfig.json"),
            r#"{ "extends": ["./first.json", "./second.json"] }"#,
        )
        .unwrap();
        let paths = super::project_paths(&root.join("src/App.vue")).unwrap();
        assert_eq!(
            paths.entries,
            vec![("#c/*".to_string(), "./src/*".to_string())]
        );
    }

    #[test]
    fn extends_cycle_keeps_the_paths_outside_the_loop() {
        let dir = temp_dir();
        let root = dir.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("a.json"), r#"{ "extends": "./b.json" }"#).unwrap();
        fs::write(
            root.join("b.json"),
            r##"{ "extends": "./a.json", "compilerOptions": { "paths": { "#c/*": ["./src/*"] } } }"##,
        )
        .unwrap();
        fs::write(root.join("tsconfig.json"), r#"{ "extends": "./a.json" }"#).unwrap();
        let paths = super::project_paths(&root.join("src/App.vue")).unwrap();
        assert_eq!(paths.anchor, root);
        assert_eq!(
            paths.entries,
            vec![("#c/*".to_string(), "./src/*".to_string())]
        );
    }

    #[test]
    fn package_extends_loads_paths_from_node_modules() {
        let dir = temp_dir();
        let root = dir.path();
        let package = root.join("node_modules/@scope/pkg");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(&package).unwrap();
        fs::write(
            package.join("tsconfig.json"),
            r##"{ "compilerOptions": { "paths": { "#c/*": ["../../src/*"] } } }"##,
        )
        .unwrap();
        fs::write(
            root.join("tsconfig.json"),
            r#"{ "extends": "@scope/pkg/tsconfig.json" }"#,
        )
        .unwrap();
        let paths = super::project_paths(&root.join("src/App.vue")).unwrap();
        assert!(paths.anchor.ends_with("node_modules/@scope/pkg"));
        assert_eq!(
            paths.entries,
            vec![("#c/*".to_string(), "../../src/*".to_string())]
        );
    }
}
