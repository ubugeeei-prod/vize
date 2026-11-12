//! Imported prop types behind a `paths` alias resolve through the project the
//! host opened, not the nearest `tsconfig.json`. reka-ui's nearest config is
//! solution-style (`files: []` plus `references`) and declares no `paths`, so
//! `PrimitiveProps` from `@/Primitive` used to resolve to nothing: `as` (bound
//! as `_as` in the template scope) became an unknown name.

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};
use std::path::PathBuf;

#[test]
#[ignore = "requires the authenticated beta host and actual source CLI in native-canon Actions"]
fn standard_tsgo_resolves_imported_props_through_the_opened_project_paths() {
    let host = std::env::var_os(TSGO_ENV)
        .map(PathBuf::from)
        .expect("this explicitly selected regression requires the real beta content-mapper host");
    let fixture = workspace_root()
        .join("tests/_fixtures/differential/typechecker/solution-tsconfig-imported-props");
    let parent = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&parent).unwrap();
    let project = tempfile::Builder::new()
        .prefix("solution-tsconfig-imported-props-")
        .tempdir_in(parent)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    std::fs::create_dir_all(project.path().join("src/Primitive")).unwrap();
    for (input, target) in [
        ("tsconfig.json.txt", "tsconfig.json"),
        ("tsconfig.app.json.txt", "tsconfig.app.json"),
        ("Primitive.ts.txt", "src/Primitive/Primitive.ts"),
        ("index.ts.txt", "src/Primitive/index.ts"),
        ("Trigger.vue.txt", "src/Trigger.vue"),
    ] {
        let source = std::fs::read(fixture.join(input)).unwrap();
        std::fs::write(project.path().join(target), source).unwrap();
    }
    std::fs::write(
        project.path().join("tsconfig.mapper.json"),
        r#"{ "extends": "./tsconfig.app.json", "contentMappers": [{ "package": "vize", "extensions": [".vue"] }] }"#,
    )
    .unwrap();

    let result = check_project(&host, project.path(), "tsconfig.mapper.json");
    // vue-tsc 3.3.14 reports exactly this row for the same project.
    assert_eq!(result.status.code(), Some(2), "{}", output_text(&result));
    assert_eq!(result.stderr, b"", "{}", output_text(&result));
    assert_eq!(
        result.stdout,
        b"src/Trigger.vue(20,43): error TS2339: Property 'toFixed' does not exist on type 'boolean'.\n",
        "{}",
        output_text(&result)
    );
}
