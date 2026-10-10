use super::super::support::{assert_success, run};
use super::support::*;
use serde_json::Value;

#[test]
fn vite_root_global_ignore_negation_and_escapes_reach_lint_discovery() {
    let project = ignore_project();
    let root = project.path();
    let (output, report) = lint_report(root, &[]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(report, oracle!("lint-ignore-syntax.json"));
    assert_configuration(root, IGNORE_SETTINGS, false, None);
}

#[cfg(feature = "glyph")]
#[test]
fn vite_root_global_ignore_negation_and_escapes_reach_formatter_discovery() {
    let project = ignore_project();
    let root = project.path();
    assert_success(&run(root, &["fmt", "--write"]));
    assert_eq!(inventory(root), oracle!("fmt-ignore-syntax.json"));
    assert_configuration(root, IGNORE_SETTINGS, false, None);
}

#[test]
fn vite_root_global_ignore_negation_and_escapes_reach_check_programs() {
    use super::super::{corsa_path, corsa_requirement, typecheck};
    use std::process::Command;
    let Some(corsa) =
        corsa_requirement::required_or_skip(corsa_path::resolve(typecheck::workspace_root()))
    else {
        return;
    };
    let project = ignore_project();
    let root = project.path();
    typecheck::link_vue(root);
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("CORSA_PATH", &corsa)
        .env("RAYON_NUM_THREADS", "1")
        .args(["check", "--format", "json"])
        .output()
        .unwrap();
    let mut report: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {output:?}"));
    normalize_report(root, &mut report);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(report, oracle!("check-ignore-syntax.json"));
    assert_configuration(root, IGNORE_SETTINGS, false, None);
}
