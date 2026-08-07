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

/// The effective `paths` map for `source_path`: the declaring config's
/// directory (targets resolve against it) and the (pattern, target) pairs
/// spelled as written.
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
    let paths = value.get("compilerOptions")?.get("paths")?.as_object()?;
    let anchor = config_path.parent()?.to_path_buf();
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
        .or_else(|| serde_json::from_str(&strip_jsonc_comments(&content)).ok())
}

fn strip_jsonc_comments(source: &str) -> std::string::String {
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
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn comments_strip_without_touching_path_patterns() {
        let source = r#"{
  // line comment
  /* block "@/decoy" comment */
  "compilerOptions": { "paths": { "@/*": ["./src/*"] } }
}"#;
        let value: serde_json::Value =
            serde_json::from_str(&super::strip_jsonc_comments(source)).unwrap();
        assert_eq!(
            value["compilerOptions"]["paths"]["@/*"][0],
            serde_json::json!("./src/*")
        );
    }
}
