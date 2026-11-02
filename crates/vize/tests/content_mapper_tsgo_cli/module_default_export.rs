//! A paired `<script>` keeps exact positions around its rewritten default
//! export (misskey MkAutocomplete), and quoted "export default" adds nothing.

use std::path::PathBuf;

use serde_json::json;

use super::{
    TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

#[test]
fn module_default_export_rewrite_keeps_authored_positions() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    let fixture = workspace_root()
        .join("tests/_fixtures/differential/typechecker/content-mapper-module-default-export");
    let root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-module-default-export-")
        .tempdir_in(root)
        .unwrap();
    let project = project.path();
    install_mapper_manifest(project);
    install_vue_package(project);
    std::fs::create_dir(project.join("src")).unwrap();
    for name in ["Picker", "Label"] {
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
            "src/Picker.vue(11,7): error TS2322: Type 'string' is not assignable to type 'number'.",
            "src/Picker.vue(12,22): error TS7006: Parameter 'value' implicitly has an 'any' type.",
        ],
        "{}",
        output_text(&output)
    );
}
