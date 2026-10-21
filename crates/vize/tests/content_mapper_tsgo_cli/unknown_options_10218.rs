use std::path::{Path, PathBuf};

use super::{
    TSGO_ENV, check_project, copy_fixture, install_mapper_manifest, install_vue_package,
    output_text, workspace_root,
};

fn require_unknown_diagnostic(output: &str, name: &str) {
    assert!(
        output.lines().any(|line| {
            line.contains("src/App.vue")
                && line.contains("TS2339")
                && line.contains(&format!("Property '{name}' does not exist"))
        }),
        "missing mapped diagnostic for {name}:\n{output}"
    );
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
    let enabled = output_text(&enabled);
    require_unknown_diagnostic(&enabled, "NotImported");
    require_unknown_diagnostic(&enabled, "vNotRegistered");

    let disabled = check_project(&tsgo, project.path(), "tsconfig.disabled.json");
    let disabled = output_text(&disabled);
    for name in ["NotImported", "vNotRegistered"] {
        assert!(
            !disabled.contains(&format!("Property '{name}' does not exist")),
            "disabled {name} check still reported:\n{disabled}"
        );
    }
}
