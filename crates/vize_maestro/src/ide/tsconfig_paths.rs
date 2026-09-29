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

#[path = "tsconfig_paths_load.rs"]
mod load;
use load::{inherited_paths, referenced_configs};

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
            serde_json::from_str(&super::load::strip_jsonc_sugar(source)).unwrap();
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
        assert_eq!(paths.anchor, package);
        assert_eq!(
            paths.entries,
            vec![("#c/*".to_string(), "../../src/*".to_string())]
        );
    }
}
