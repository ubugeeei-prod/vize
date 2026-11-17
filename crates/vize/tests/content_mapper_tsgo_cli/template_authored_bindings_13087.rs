//! #13087: authored `<script setup>` bindings win over template-context names
//! (`props`, `$attrs`, `$emit`) under the content mapper, as under `vize check`.
#![expect(clippy::disallowed_methods, reason = "fixture IO uses std strings")]

use std::path::PathBuf;

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

pub(crate) const FIXTURE: &str =
    "tests/_fixtures/differential/typechecker/template-authored-bindings-13087";

pub(crate) const INPUTS: [&str; 6] = [
    "tsconfig.json",
    "src/PropsName.vue",
    "src/PropsObject.vue",
    "src/ContextMembers.vue",
    "src/PropsControl.vue",
    "src/PropsRest.vue",
];

/// Only the true positives remain: a missing member of the props object
/// itself, and a member the rest object omits.
pub(crate) const EXPECTED: [(&str, u32, u32, &str); 2] = [
    (
        "src/PropsControl.vue",
        6,
        42,
        "Property 'missing' does not exist on type",
    ),
    (
        "src/PropsRest.vue",
        6,
        59,
        "Property 'title' does not exist on type",
    ),
];

#[test]
fn authored_template_bindings_resolve_under_the_content_mapper() {
    let Some(native) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping authored binding conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(native.is_file(), "{TSGO_ENV} must name an executable");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-authored-bindings-")
        .tempdir_in(cases_root)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    let fixture = workspace_root().join(FIXTURE);
    for name in INPUTS {
        let target = project.path().join(name);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(fixture.join(vize_l0::cstr!("{name}.txt").as_str()), target).unwrap();
    }
    let mut tsconfig: serde_json::Value =
        serde_json::from_slice(&std::fs::read(project.path().join("tsconfig.json")).unwrap())
            .unwrap();
    tsconfig["contentMappers"] = serde_json::json!([{ "package": "vize", "extensions": [".vue"] }]);
    std::fs::write(
        project.path().join("tsconfig.json"),
        serde_json::to_vec_pretty(&tsconfig).unwrap(),
    )
    .unwrap();

    let output = check_project(&native, project.path(), "tsconfig.json");
    assert_eq!(output.status.code(), Some(2), "{}", output_text(&output));
    assert_eq!(output.stderr, b"", "{}", output_text(&output));
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let observed = stdout
        .lines()
        .map(|line| {
            let (location, message) = line.split_once(": error TS2339: ").unwrap();
            let (file, position) = location.split_once('(').unwrap();
            let (line, column) = position.trim_end_matches(')').split_once(',').unwrap();
            let message = message.split(" on type").next().unwrap();
            (
                file,
                line.parse::<u32>().unwrap(),
                column.parse::<u32>().unwrap(),
                vize_l0::cstr!("{message} on type"),
            )
        })
        .collect::<Vec<_>>();
    let expected = EXPECTED
        .iter()
        .map(|(file, line, column, message)| (*file, *line, *column, vize_l0::cstr!("{message}")))
        .collect::<Vec<_>>();
    assert_eq!(observed, expected, "{}", output_text(&output));
}
