use super::{
    corsa_path, corsa_requirement,
    support::{self, assert_no_dedicated_config, project, write},
};
use serde_json::Value;
use std::{fs, path::Path, process::Command};

pub(super) fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
}

pub(super) fn link_vue(root: &Path) {
    let workspace = workspace_root();
    let vue = [
        "npm/cli/node_modules/vue",
        "node_modules/vue",
        "tests/node_modules/vue",
        "examples/vite-musea/node_modules/vue",
    ]
    .into_iter()
    .map(|name| workspace.join(name))
    .find(|path| path.exists())
    .expect("Vue fixture dependency must be installed");
    let vue = fs::canonicalize(vue).unwrap();
    fs::create_dir_all(root.join("node_modules")).unwrap();
    let namespace = vue.parent().unwrap().join("@vue");
    link(&vue, &root.join("node_modules/vue"));
    if namespace.exists() {
        link(&namespace, &root.join("node_modules/@vue"));
    }
}

fn link(source: &Path, target: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
}

fn check(root: &Path, corsa: &str) -> (std::process::Output, Value) {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("CORSA_PATH", corsa)
        .env("RAYON_NUM_THREADS", "1")
        .args(["check", "--format", "json"])
        .output()
        .unwrap();
    let report = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {output:?}"));
    (output, support::normalized_report(root, report))
}

#[test]
fn tsconfig_aliases_preserve_valid_types_and_expose_invalid_props_and_ts() {
    let Some(corsa) = corsa_requirement::required_or_skip(corsa_path::resolve(workspace_root()))
    else {
        return;
    };
    let project = project();
    let root = project.path();
    link_vue(root);
    let (valid, report) = check(root, &corsa);
    assert!(valid.status.success(), "{valid:?}\n{report:#}");
    insta::assert_snapshot!(
        "project_settings_check_valid",
        serde_json::to_string_pretty(&report).unwrap()
    );
    let valid_report = report;

    write(
        root,
        "src/Consumer.vue",
        &support::CONSUMER.replace(":count=\"count\"", ":count=\"'wrong'\""),
    );
    write(root, "src/invalid.ts", support::INVALID_TS);
    let (invalid, report) = check(root, &corsa);
    assert_eq!(invalid.status.code(), Some(1), "{invalid:?}\n{report:#}");
    insta::assert_snapshot!(
        "project_settings_check_invalid",
        serde_json::to_string_pretty(&report).unwrap()
    );

    write(root, "src/Consumer.vue", support::CONSUMER);
    fs::remove_file(root.join("src/invalid.ts")).unwrap();
    let (repaired, report) = check(root, &corsa);
    assert!(repaired.status.success(), "{repaired:?}\n{report:#}");
    assert_eq!(report, valid_report);
    assert_no_dedicated_config(root);
}
