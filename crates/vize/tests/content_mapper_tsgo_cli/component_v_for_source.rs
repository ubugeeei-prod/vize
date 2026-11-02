//! A component `v-for` source is replayed for prop checks; only the template
//! copy is mapped, so its diagnostics report once (misskey roles.vue).

use std::path::PathBuf;

use serde_json::json;

use super::{
    TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

#[test]
fn component_v_for_source_reports_once() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    let fixture = workspace_root()
        .join("tests/_fixtures/differential/typechecker/content-mapper-component-v-for-source");
    let root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-component-v-for-source-")
        .tempdir_in(root)
        .unwrap();
    let project = project.path();
    install_mapper_manifest(project);
    install_vue_package(project);
    std::fs::create_dir(project.join("src")).unwrap();
    for name in ["Child", "Rows"] {
        std::fs::copy(
            fixture.join(cstr!("{name}.vue.txt").as_str()),
            project.join(cstr!("src/{name}.vue").as_str()),
        )
        .unwrap();
    }
    std::fs::write(
        project.join("tsconfig.json"),
        serde_json::to_vec_pretty(&json!({
            "compilerOptions": {
                "strict": true, "noEmit": true, "skipLibCheck": true,
                "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler",
                "types": [], "lib": ["ES2022", "DOM"]
            },
            "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
            "include": ["src"]
        }))
        .unwrap(),
    )
    .unwrap();
    let output = check_project(&tsgo, project, "tsconfig.json");
    let mut lines = std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    lines.sort_unstable();
    assert_eq!(
        lines,
        [
            "src/Rows.vue(6,36): error TS7006: Parameter 'x' implicitly has an 'any' type.",
            "src/Rows.vue(8,43): error TS2322: Type 'string' is not assignable to type 'number'.",
        ],
        "{}",
        output_text(&output)
    );
}
