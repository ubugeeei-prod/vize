//! TypeScript only treats `*`, `?` and `**` as wildcards in `include` and
//! `exclude`; file-based routing names like `[id].vue` are literal (#10819).
//! Expectations match `tsc -p` and `tsgo -p --listFilesOnly` on the same tree.

#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use std::path::{Path, PathBuf};

use super::{TsconfigInputCache, normalize_input_path, resolve_tsconfig_for_files};

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn project(tsconfig: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let root = normalize_input_path(dir.path());
    for file in [
        "src/[id].vue",
        "src/i.vue",
        "src/d.vue",
        "src/{a,b}.ts",
        "src/a.ts",
        "src/!x.ts",
        "src/x.ts",
        "src/pages/[legacy]/index.vue",
        "src/pages/l/index.vue",
    ] {
        write(&root, file, "<template />");
    }
    write(&root, "tsconfig.json", tsconfig);
    (dir, root)
}

fn collected(root: &Path) -> Vec<String> {
    super::collect_default_check_files(
        root,
        Some(&root.join("tsconfig.json")),
        false,
        &mut TsconfigInputCache::default(),
    )
    .iter()
    .map(|path| {
        path.strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/")
    })
    .collect()
}

#[test]
fn brackets_braces_and_bangs_in_includes_are_literal() {
    let (_dir, root) = project(r#"{"include":["src/[id].vue","src/{a,b}.ts","src/!x.ts"]}"#);
    assert_eq!(
        collected(&root),
        vec!["src/!x.ts", "src/[id].vue", "src/{a,b}.ts"]
    );
}

#[test]
fn a_bracket_directory_is_an_implicit_glob_for_include_and_exclude() {
    let (_dir, root) = project(r#"{"include":["src/pages/[legacy]"]}"#);
    assert_eq!(collected(&root), vec!["src/pages/[legacy]/index.vue"]);

    let (_dir, root) = project(r#"{"include":["src/pages"],"exclude":["src/pages/[legacy]"]}"#);
    assert_eq!(collected(&root), vec!["src/pages/l/index.vue"]);
}

#[test]
fn a_bracket_page_belongs_to_the_project_that_names_it() {
    let (_dir, root) =
        project(r#"{"files":[],"references":[{"path":"./app.json"},{"path":"./other.json"}]}"#);
    write(&root, "app.json", r#"{"include":["src/[id].vue"]}"#);
    write(&root, "other.json", r#"{"include":["src/i.vue"]}"#);

    let owner = |file: &str| {
        resolve_tsconfig_for_files(
            Some(&root.join("tsconfig.json")),
            &[root.join(file)],
            false,
            &mut TsconfigInputCache::default(),
        )
        .unwrap()
    };
    assert_eq!(owner("src/[id].vue"), root.join("app.json"));
    assert_eq!(owner("src/i.vue"), root.join("other.json"));
    assert_eq!(owner("src/d.vue"), root.join("tsconfig.json"));
}
