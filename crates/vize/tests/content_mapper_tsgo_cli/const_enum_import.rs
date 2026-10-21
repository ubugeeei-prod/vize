//! Pin imported const-enum setup anchors against the exact Content Mapper host.

use std::path::PathBuf;

use serde_json::json;

use super::{
    TSGO_ENV, assert_success, check_project, install_mapper_manifest, install_vue_package,
    output_text, workspace_root,
};

fn fixture_root() -> PathBuf {
    workspace_root()
        .join("tests/_fixtures/differential/typechecker/content-mapper-const-enum-import")
}

#[test]
fn standard_tsgo_accepts_imported_const_enum_setup_bindings() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");

    let vue = std::fs::read_to_string(fixture_root().join("App.vue.txt")).unwrap();
    let state = std::fs::read_to_string(fixture_root().join("state.ts.txt")).unwrap();
    for newline in ["\n", "\r\n"] {
        let cases_root = workspace_root().join("target/vize-tests/tests");
        std::fs::create_dir_all(&cases_root).unwrap();
        let project = tempfile::Builder::new()
            .prefix("content-mapper-const-enum-import-")
            .tempdir_in(cases_root)
            .unwrap();
        install_mapper_manifest(project.path());
        install_vue_package(project.path());
        std::fs::create_dir(project.path().join("src")).unwrap();
        std::fs::write(
            project.path().join("src/App.vue"),
            vue.replace('\n', newline),
        )
        .unwrap();
        std::fs::write(
            project.path().join("src/state.ts"),
            state.replace('\n', newline),
        )
        .unwrap();
        std::fs::write(
            project.path().join("tsconfig.json"),
            serde_json::to_vec_pretty(&json!({
                "compilerOptions": {
                    "strict": true, "noEmit": true, "skipLibCheck": true,
                    "target": "ES2022", "module": "ESNext",
                    "moduleResolution": "bundler", "types": []
                },
                "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
                "include": ["src"]
            }))
            .unwrap(),
        )
        .unwrap();

        let output = check_project(&tsgo, project.path(), "tsconfig.json");
        assert_success(&output);
        assert!(output.stdout.is_empty(), "{}", output_text(&output));
        assert!(output.stderr.is_empty(), "{}", output_text(&output));
    }
}
