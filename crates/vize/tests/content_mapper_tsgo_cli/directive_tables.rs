//! Directive tables stay valid: sibling expects never overlap (the host would
//! refuse the whole file), and unmapped scaffolding is ignored like Vue does.

use std::path::PathBuf;

use serde_json::json;

use super::{
    TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

#[test]
fn directive_tables_keep_files_checked_and_hide_only_unmapped_scaffolding() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    let fixture = workspace_root()
        .join("tests/_fixtures/differential/typechecker/content-mapper-directive-tables");
    let root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-directive-tables-")
        .tempdir_in(root)
        .unwrap();
    let project = project.path();
    install_mapper_manifest(project);
    install_vue_package(project);
    std::fs::create_dir(project.join("src")).unwrap();
    for name in ["Child", "Siblings", "Partial", "Unknown", "Kebab"] {
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
        .map(str::trim)
        .collect::<Vec<_>>();
    lines.sort_unstable();
    assert_eq!(
        lines,
        [
            "src/Kebab.vue(3,7): error TS2322: Type 'string' is not assignable to type 'number'.",
            "src/Partial.vue(5,8): error vize4: Unused '@vue-expect-error' directive",
            "src/Partial.vue(9,11): error TS2322: Type 'string' is not assignable to type 'number'.",
            "src/Unknown.vue(6,8): error vize4: Unused '@vue-expect-error' directive",
        ],
        "{}",
        output_text(&output)
    );
}
