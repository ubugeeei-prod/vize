//! Shared `tsconfig.json` `paths` reading for editor features (#3915, #3917).
//!
//! Anchoring matches the session's rule: the nearest `tsconfig.json` governs;
//! when it is a solution-style shell that declares no `paths` of its own, the
//! first referenced project config that does wins (the create-vue app/node
//! split has exactly one). Comment stripping is string-aware — every `paths`
//! pattern contains `/*` (`"@/*"`), so a stripper that ignores string state
//! destroys exactly the value these features need.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

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
    if let Some(paths) = paths_of(&shell) {
        return Some(paths);
    }
    referenced_configs(&shell)
        .into_iter()
        .find_map(|referenced| paths_of(&referenced))
}

fn paths_of(config_path: &Path) -> Option<ProjectPaths> {
    let value = read_jsonc(config_path)?;
    let compiler_options = value.get("compilerOptions")?;
    let paths = compiler_options.get("paths")?.as_object()?;
    let config_dir = config_path.parent()?;
    // TypeScript resolves `paths` targets against `baseUrl` when it is set,
    // falling back to the declaring config's directory otherwise.
    let anchor = match compiler_options.get("baseUrl").and_then(|v| v.as_str()) {
        Some(base_url) => config_dir.join(base_url),
        None => config_dir.to_path_buf(),
    };
    let mut entries = Vec::new();
    for (pattern, targets) in paths {
        for target in targets.as_array().into_iter().flatten() {
            if let Some(target) = target.as_str() {
                entries.push((pattern.clone(), target.to_owned()));
            }
        }
    }
    (!entries.is_empty()).then_some(ProjectPaths { anchor, entries })
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
}
