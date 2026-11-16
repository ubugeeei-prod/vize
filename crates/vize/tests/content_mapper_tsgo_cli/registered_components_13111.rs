//! #13111: a key of the `components` option is a known component typed by its value.

use std::path::{Path, PathBuf};

use super::{
    TSGO_ENV, check_project, copy_fixture, install_mapper_manifest, install_vue_package,
    output_text, workspace_root,
};

const PRECISION: [&str; 4] = [
    "src/MixedWrong.vue(14,13): error TS2322: Type 'number' is not assignable to type 'string'.",
    "src/Wrong.vue(13,13): error TS2322: Type 'number' is not assignable to type 'string'.",
    "src/Wrong.vue(14,17): error TS2322: Type 'number' is not assignable to type 'string'.",
    "src/Wrong.vue(15,18): error TS2322: Type 'number' is not assignable to type 'string'.",
];
const UNKNOWN: [&str; 2] = [
    "src/MixedWrong.vue(15,4): error TS2339: Property 'NotRegistered' does not exist on type '{}'.",
    "src/Wrong.vue(16,4): error TS2339: Property 'NotRegistered' does not exist on type '{}'.",
];

fn diagnostic_lines(output: &std::process::Output) -> Vec<std::string::String> {
    assert!(output.stderr.is_empty(), "{}", output_text(output));
    let mut lines: Vec<_> = std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.replace('\\', "/"))
        .collect();
    lines.sort_unstable();
    lines
}

#[test]
fn registered_component_keys_are_known_and_typed_by_their_values() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping registered component conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(
        tsgo.is_file(),
        "{TSGO_ENV} is not a file: {}",
        tsgo.display()
    );

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/content_mapper_registered_components_13111");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-registered-components-13111-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_mapper_manifest(project.path());
    install_vue_package(project.path());

    // Registered.vue is the reported project; Shapes.vue and Mixed.vue cover
    // `.ts`, library, member, kebab, string-key, async and `<script setup>`
    // registrations. Only the unregistered tags stay unknown.
    let mut expected: Vec<_> = PRECISION.iter().chain(&UNKNOWN).copied().collect();
    expected.sort_unstable();
    let enabled = check_project(&tsgo, project.path(), "tsconfig.json");
    assert_eq!(diagnostic_lines(&enabled), expected);

    let disabled = check_project(&tsgo, project.path(), "tsconfig.disabled.json");
    assert_eq!(diagnostic_lines(&disabled), PRECISION);
}
