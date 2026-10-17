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

#[test]
fn inherited_vue_targets_preserve_complete_checker_reports_and_vite_precedence() {
    let Some(corsa) = corsa_requirement::required_or_skip(corsa_path::resolve(workspace_root()))
    else {
        return;
    };
    let project = project();
    let root = project.path();
    link_vue(root);
    write(
        root,
        "src/Consumer.vue",
        &support::CONSUMER.replace(":count=\"count\"", ":count=\"'wrong'\""),
    );
    write(root, "src/invalid.ts", support::INVALID_TS);
    let (baseline, baseline_report) = check(root, &corsa);
    assert_eq!(baseline.status.code(), Some(1), "{baseline:?}");
    assert!(baseline_report["errorCount"].as_u64().unwrap() >= 2);
    let authored_config: Value =
        serde_json::from_slice(&fs::read(root.join("tsconfig.json")).unwrap()).unwrap();
    // TypeScript options remain declared by tsconfig. Vue target inheritance
    // must not replace paths, strictness, or the diagnostic source positions.
    for target in [
        serde_json::json!(3),
        serde_json::json!(3.5),
        serde_json::json!(99),
        serde_json::json!("auto"),
    ] {
        let mut base = authored_config.clone();
        base.as_object_mut().unwrap().remove("include");
        base["vueCompilerOptions"] = serde_json::json!({"target":target});
        fs::write(
            root.join("tsconfig.base.json"),
            serde_json::to_vec(&base).unwrap(),
        )
        .unwrap();
        write(
            root,
            "tsconfig.json",
            "{\n// Native JSONC project inheritance\n\"extends\": \"./tsconfig.base.json\",\n\"include\": [\"src/**/*\"],\n}\n",
        );
        let (actual, report) = check(root, &corsa);
        assert_eq!(actual.status.code(), Some(1), "{target}: {actual:?}");
        assert_eq!(report, baseline_report, "{target}: complete checker report");
    }

    // An unused solution-shell target must not override its owning referenced
    // project's target, even when the shell value is unsupported.
    fs::rename(root.join("tsconfig.json"), root.join("tsconfig.app.json")).unwrap();
    write(
        root,
        "tsconfig.json",
        r#"{
        "files": [], "references": [{"path":"./tsconfig.app.json"}],
        "vueCompilerOptions": {"target":4}
    }"#,
    );
    let (referenced, referenced_report) = check(root, &corsa);
    assert_eq!(referenced.status.code(), Some(1), "{referenced:?}");
    let mut expected_reference = baseline_report.clone();
    expected_reference["programs"][0]["tsconfig"] = serde_json::json!("tsconfig.app.json");
    assert_eq!(
        referenced_report, expected_reference,
        "complete referenced program report"
    );

    // Unsupported target values fail with an authored configuration error,
    // before a checker can silently interpret the project as Vue 3.
    write(
        root,
        "tsconfig.base.json",
        r#"{
        "vueCompilerOptions": {"target":4}
    }"#,
    );
    let invalid = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("CORSA_PATH", &corsa)
        .env("NO_COLOR", "1")
        .env_remove("FORCE_COLOR")
        .args(["check", "--format", "json"])
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(1), "{invalid:?}");
    assert!(invalid.stdout.is_empty(), "{invalid:?}");
    assert_eq!(
        std::str::from_utf8(&invalid.stderr).unwrap(),
        vize_l0::cstr!(
            "Error: Configuration error: unsupported vueCompilerOptions.target 4 in {}; expected a numeric Vue 2.x target up to 2.7, a Vue 3.x target, 99, or \"auto\"\n",
            root.canonicalize().unwrap().join("tsconfig.app.json").display()
        ).as_str(),
        "{invalid:?}"
    );

    // Restore exactly the same TypeScript options and deliberately retain the
    // unsupported target: explicit Vite Vue 3 wins over the tsconfig fallback.
    let mut invalid_target = authored_config;
    invalid_target.as_object_mut().unwrap().remove("include");
    invalid_target["vueCompilerOptions"] = serde_json::json!({"target":4});
    fs::write(
        root.join("tsconfig.base.json"),
        serde_json::to_vec(&invalid_target).unwrap(),
    )
    .unwrap();
    write(
        root,
        "vite.config.mjs",
        &support::SETTINGS.replace("vue: {", "vue: { version: '3',"),
    );
    let (overridden, overridden_report) = check(root, &corsa);
    assert_eq!(overridden.status.code(), Some(1), "{overridden:?}");
    assert_eq!(
        overridden_report, expected_reference,
        "explicit Vite Vue 3 complete report"
    );
    assert_no_dedicated_config(root);
}
