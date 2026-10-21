//! Pin inferred type-only imports against the exact Content Mapper host.

use std::path::PathBuf;

use serde_json::json;

use super::{
    TSGO_ENV, assert_success, check_project, install_mapper_manifest, install_vue_package,
    output_text, workspace_root,
};

fn fixture_root() -> PathBuf {
    workspace_root()
        .join("tests/_fixtures/differential/typechecker/content-mapper-type-only-import-use")
}

#[test]
fn standard_tsgo_accepts_imports_that_resolve_only_as_types() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");

    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-type-only-import-use-")
        .tempdir_in(cases_root)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    std::fs::create_dir(project.path().join("src")).unwrap();
    for (source, destination) in [
        ("model.ts.txt", "model.ts"),
        ("List.vue.txt", "List.vue"),
        ("Unused.vue.txt", "Unused.vue"),
        ("Annotated.vue.txt", "Annotated.vue"),
    ] {
        std::fs::copy(
            fixture_root().join(source),
            project.path().join("src").join(destination),
        )
        .unwrap();
    }
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
