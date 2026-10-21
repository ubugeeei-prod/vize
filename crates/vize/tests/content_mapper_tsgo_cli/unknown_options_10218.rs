use std::path::{Path, PathBuf};

use super::{
    TSGO_ENV, assert_success, check_project, copy_fixture, install_mapper_manifest,
    install_vue_package, output_text, workspace_root,
};

fn diagnostic_lines(output: &std::process::Output) -> Vec<std::string::String> {
    assert!(output.stderr.is_empty(), "{}", output_text(output));
    std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.replace('\\', "/"))
        .collect()
}

#[test]
fn configured_unknown_component_and_directive_match_vue_diagnostics() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper unknown-option parity: {TSGO_ENV} is not set");
        return;
    };
    assert!(
        tsgo.is_file(),
        "{TSGO_ENV} is not a file: {}",
        tsgo.display()
    );

    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/content_mapper_unknown_options_10218");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-unknown-options-10218-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_mapper_manifest(project.path());
    install_vue_package(project.path());

    let enabled = check_project(&tsgo, project.path(), "tsconfig.json");
    assert!(!enabled.status.success(), "fixture passed unexpectedly");
    assert_eq!(
        diagnostic_lines(&enabled),
        vec![
            "src/App.vue(7,6): error TS2339: Property 'NotImported' does not exist on type '{}'.",
            "src/App.vue(8,8): error TS2339: Property 'vNotRegistered' does not exist on type '{}'.",
        ]
    );

    let disabled = check_project(&tsgo, project.path(), "tsconfig.disabled.json");
    assert_success(&disabled);
    assert_eq!(
        diagnostic_lines(&disabled),
        Vec::<std::string::String>::new()
    );
}
