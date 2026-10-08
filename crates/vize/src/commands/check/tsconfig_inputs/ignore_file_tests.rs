//! A tsconfig selects authored roots independently of filesystem ignore files.

use std::path::{Path, PathBuf};

use serde_json::Value;
use vize_l0::cstr;

use super::{TsconfigInputCache, collect_ambient_declaration_files, collect_default_check_files};

const INPUT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/tsconfig-ignore-3984/input.json"
));

fn project(name: &str, corpus: &Value) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let root = std::env::temp_dir()
        .join(cstr!("vize-config-ignore-{name}-{}-{id}", std::process::id()).as_str());
    assert!(!root.exists());
    for files in [&corpus["commonFiles"], &corpus["cases"][name]["files"]] {
        for (name, bytes) in files.as_object().unwrap() {
            let path = root.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, bytes.as_str().unwrap()).unwrap();
        }
    }
    std::fs::canonicalize(root).unwrap()
}

fn relative(root: &Path, files: Vec<PathBuf>) -> Vec<String> {
    files
        .into_iter()
        .map(|file| {
            file.strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

#[test]
fn configured_roots_keep_gitignore_ignore_and_git_exclude_inputs_and_ambient_types() {
    let corpus: Value = serde_json::from_str(INPUT).unwrap();
    for (name, case) in corpus["cases"].as_object().unwrap() {
        let root = project(name, &corpus);
        let config = root.join("tsconfig.json");
        let mut cache = TsconfigInputCache::default();
        let files = relative(
            &root,
            collect_default_check_files(&root, Some(&config), false, &mut cache),
        );
        let expected = case["reportedFiles"]
            .as_array()
            .unwrap()
            .iter()
            .map(|file| file.as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(files, expected, "whole {name} root selection");
        assert_eq!(
            relative(
                &root,
                collect_ambient_declaration_files(&root, Some(&config), &mut cache)
            ),
            ["src/toolignored/globals.d.ts"],
            "whole {name} explicit-run ambient selection"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn unconfigured_discovery_keeps_filesystem_ignore_rules() {
    let corpus: Value = serde_json::from_str(INPUT).unwrap();
    let root = project("direct", &corpus);
    assert_eq!(
        relative(
            &root,
            collect_default_check_files(&root, None, false, &mut TsconfigInputCache::default())
        ),
        ["src/entry.ts", "src/excluded/decoy.ts"]
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn configured_walk_prunes_wildcard_packages_but_retains_literal_package_paths() {
    use super::{collect::skip_implicit_package_directory, spec::GlobSpec};
    let root = Path::new("/fixture");
    let wildcard = [GlobSpec::new(root, "**/*").unwrap()];
    for name in ["node_modules", "bower_components", "jspm_packages"] {
        assert!(skip_implicit_package_directory(&root.join(name), &wildcard));
        assert!(skip_implicit_package_directory(
            &root.join("packages/a").join(name),
            &wildcard
        ));
        let literal = [GlobSpec::new(root, &cstr!("packages/*/{name}/selected/*.ts")).unwrap()];
        assert!(!skip_implicit_package_directory(
            &root.join("packages/a").join(name),
            &literal
        ));
        let uppercase = [GlobSpec::new(
            root,
            &cstr!("packages/*/{}/selected/*.ts", name.to_ascii_uppercase()),
        )
        .unwrap()];
        assert_eq!(
            skip_implicit_package_directory(&root.join("packages/a").join(name), &uppercase),
            !cfg!(windows),
            "literal package retention follows the platform glob case policy"
        );
        let anchored = [GlobSpec::new(&root.join(name).join("selected"), "**/*").unwrap()];
        assert!(!skip_implicit_package_directory(
            &root.join(name),
            &anchored
        ));
    }
    assert!(!skip_implicit_package_directory(
        &root.join("src"),
        &wildcard
    ));
}

#[test]
fn directory_pruning_preserves_original_generated_root_and_codegen_declaration_selection() {
    use super::{
        collect::collect_supported_files_with_options,
        spec::{FileCollectionOptions, GlobSpec},
    };
    let corpus: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/tsconfig-ignore-3984/generated-boundaries.json"
    ))).unwrap();
    let project = std::env::temp_dir()
        .join(cstr!("vize-generated-ignore-control-{}", std::process::id()).as_str());
    assert!(!project.exists());
    for (name, bytes) in corpus["files"].as_object().unwrap() {
        let path = project.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes.as_str().unwrap()).unwrap();
    }
    let project = std::fs::canonicalize(project).unwrap();
    for (name, case) in corpus["cases"].as_object().unwrap() {
        let root = project.join(case["relativeRoot"].as_str().unwrap());
        let root = std::fs::canonicalize(root).unwrap();
        let includes = case["includes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|pattern| GlobSpec::new(&root, pattern.as_str().unwrap()).unwrap())
            .collect::<Vec<_>>();
        let files = relative(
            &root,
            collect_supported_files_with_options(
                &root,
                &includes,
                &[],
                FileCollectionOptions::default(),
            ),
        );
        let expected = case["expected"]
            .as_array()
            .unwrap()
            .iter()
            .map(|file| file.as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(files, expected, "whole original generated selection {name}");
    }
    std::fs::remove_dir_all(project).unwrap();
}
