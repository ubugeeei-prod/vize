//! Path references own the complete declaration graph in production scans.
#![expect(clippy::unwrap_used, reason = "fixture assertions panic")]

use vize_canon::BatchTypeChecker;

#[path = "support/reference_path_project.rs"]
mod project;

#[test]
fn reported_project_matches_the_pinned_corpus_bytes() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/typechecker/reference-path-module");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
    for name in manifest["inputs"].as_array().unwrap() {
        let name = name.as_str().unwrap();
        let bytes = std::fs::read(fixture.join(name)).unwrap();
        assert_eq!(
            project::digest(&bytes).as_str(),
            manifest["sha256"][name].as_str().unwrap()
        );
    }
}

#[test]
fn unprefixed_references_keep_the_complete_declaration_graph() {
    for case in project::CASES {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let paths = project::prepare(&root, case, false);
        let originals = paths
            .iter()
            .map(|path| std::fs::read(path).unwrap())
            .collect::<Vec<_>>();
        let mut checker = BatchTypeChecker::new(&root).unwrap();
        checker.scan_project().unwrap();
        let files = checker.virtual_files();
        for path in paths.iter().skip(1) {
            assert!(
                files.iter().any(|file| &file.original_path == path),
                "{} lost {path:?}",
                case.0
            );
        }
        for (path, original) in paths.iter().zip(originals) {
            assert_eq!(std::fs::read(path).unwrap(), original);
        }
    }
}

#[test]
fn path_references_do_not_resolve_through_import_aliases() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    project::prepare(&root, &project::CASES[0], false);
    std::fs::write(
        root.join("env.d.ts"),
        "/// <reference path=\"missing.d.ts\" />\nexport {};\n",
    )
    .unwrap();
    let config_path = root.join("tsconfig.json");
    let mut config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&config_path).unwrap()).unwrap();
    config["compilerOptions"]["paths"] =
        serde_json::json!({"missing.d.ts":["./types/builder-env.d.ts"]});
    std::fs::write(config_path, serde_json::to_vec(&config).unwrap()).unwrap();
    let mut checker = BatchTypeChecker::new(&root).unwrap();
    checker.scan_project().unwrap();
    assert!(
        checker
            .virtual_files()
            .iter()
            .all(|file| file.original_path != root.join("types/builder-env.d.ts"))
    );
}
