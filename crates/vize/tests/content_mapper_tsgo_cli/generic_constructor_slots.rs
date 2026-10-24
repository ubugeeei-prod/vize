//! The real Content Mapper host must preserve generic constructor slots (#10416).

use std::path::PathBuf;
use vize_l0::cstr;

use super::{
    TSGO_ENV, assert_success, check_project, install_mapper_manifest, install_vue_package,
    output_text, workspace_root,
};

#[test]
fn generic_constructor_slots_match_authored_member_diagnostics() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper constructor slots: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} must name an executable");
    let fixture = workspace_root()
        .join("tests/_fixtures/differential/typechecker/generic-constructor-slots-10416");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-generic-constructor-slots-")
        .tempdir_in(cases_root)
        .unwrap();
    std::fs::create_dir_all(project.path().join("src")).unwrap();
    for name in ["GenericList.ts", "Plain.vue", "Invalid.vue"] {
        std::fs::copy(
            fixture.join("src").join(cstr!("{name}.txt").as_str()),
            project.path().join("src").join(name),
        )
        .unwrap();
    }
    let mut config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fixture.join("tsconfig.json.txt")).unwrap()).unwrap();
    config["contentMappers"] = serde_json::json!([{ "package": "vize", "extensions": [".vue"] }]);
    std::fs::write(
        project.path().join("tsconfig.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());

    let invalid = check_project(&tsgo, project.path(), "tsconfig.json");
    assert_eq!(invalid.status.code(), Some(2), "{}", output_text(&invalid));
    assert!(invalid.stderr.is_empty(), "{}", output_text(&invalid));
    let lines = std::str::from_utf8(&invalid.stdout)
        .unwrap()
        .lines()
        .map(|line| line.replace('\\', "/"))
        .collect::<Vec<_>>();
    assert_eq!(
        lines,
        [
            "src/Invalid.vue(6,57): error TS2339: Property 'nope' does not exist on type '{ id: number; name: string; }'.",
            "src/Invalid.vue(7,70): error TS2339: Property 'nope' does not exist on type '{ id: number; name: string; }'.",
        ]
    );

    std::fs::remove_file(project.path().join("src/Invalid.vue")).unwrap();
    let valid = check_project(&tsgo, project.path(), "tsconfig.json");
    assert_success(&valid);
    assert!(valid.stdout.is_empty(), "{}", output_text(&valid));
    assert!(valid.stderr.is_empty(), "{}", output_text(&valid));
}
