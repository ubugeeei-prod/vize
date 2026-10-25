#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use std::{fs, path::Path, process::Command, time::Instant};
use vize_doctor::{DoctorCategory, DoctorReport};

const BROWSER_SOURCE: &str = "<script setup>\nwindow.innerWidth;\n</script>\n<template><div>{{ document.title }}</div></template>";

#[test]
fn nuxt_client_only_configuration_applies_to_complete_doctor_and_lint_packets() {
    let directory = tempfile::tempdir().unwrap();
    project(
        directory.path(),
        "export default defineNuxtConfig({ ssr: false })",
        false,
    );
    let (doctor, lint, cross_file) = packets(directory.path(), "client-only");
    assert_eq!(
        serde_json::to_value(&doctor).unwrap(),
        serde_json::to_value(DoctorReport::new(".", [])).unwrap()
    );
    let healthy_lint =
        serde_json::json!([{"file":"src/App.vue","messages":[],"errorCount":0,"warningCount":0}]);
    assert_eq!(lint, healthy_lint);
    assert_eq!(cross_file, healthy_lint);
}

#[test]
fn ssr_and_unknown_configuration_keep_real_browser_diagnostics() {
    for (name, source, vite_client_build) in [
        (
            "ssr-enabled",
            "export default defineNuxtConfig({ssr:true})",
            false,
        ),
        (
            "ssr-dynamic",
            "export default defineNuxtConfig({ssr:process.env.SSR})",
            false,
        ),
        (
            "ssr-spread",
            "export default defineNuxtConfig({ssr:false,...options})",
            false,
        ),
        (
            "ssr-environment",
            "export default defineNuxtConfig({ssr:false,$production:{ssr:true}})",
            false,
        ),
        (
            "ssr-nested",
            "export default defineNuxtConfig({app:{ssr:false}})",
            false,
        ),
        (
            "ssr-comment",
            "// ssr:false\nexport default defineNuxtConfig({})",
            false,
        ),
        ("vite-client-build", "", true),
    ] {
        let directory = tempfile::tempdir().unwrap();
        project(directory.path(), source, vite_client_build);
        let (doctor, lint, cross_file) = packets(directory.path(), name);
        assert_eq!(
            doctor
                .findings()
                .iter()
                .map(|finding| finding.code.as_str())
                .collect::<Vec<_>>(),
            [
                "VIZE_DOCTOR_CF_BROWSER_API_SSR",
                "VIZE_DOCTOR_CF_BROWSER_API_SSR"
            ],
            "{name}"
        );
        assert_eq!(doctor.summary().counts.warnings, 2, "{name}");
        assert_eq!(
            doctor
                .summary()
                .categories
                .get(&DoctorCategory::ProductionReadiness)
                .unwrap()
                .score,
            76,
            "{name}"
        );
        assert_eq!(doctor.summary().overall_score, 96, "{name}");
        assert_eq!(
            rule_ids(&lint),
            [
                "ssr/no-browser-globals-in-ssr",
                "ssr/no-browser-globals-in-ssr"
            ],
            "{name}"
        );
        let mut cross_ids = rule_ids(&cross_file);
        cross_ids.sort_unstable();
        assert_eq!(
            cross_ids,
            [
                "cross-file",
                "cross-file",
                "ssr/no-browser-globals-in-ssr",
                "ssr/no-browser-globals-in-ssr"
            ],
            "{name}"
        );
    }
}

#[test]
fn the_last_explicit_false_after_a_spread_is_authoritative() {
    let directory = tempfile::tempdir().unwrap();
    project(
        directory.path(),
        "export default defineNuxtConfig({...options, ssr:false})",
        false,
    );
    let (doctor, lint, cross_file) = packets(directory.path(), "last-literal-false");
    assert_eq!(
        serde_json::to_value(&doctor).unwrap(),
        serde_json::to_value(DoctorReport::new(".", [])).unwrap()
    );
    let healthy_lint =
        serde_json::json!([{"file":"src/App.vue","messages":[],"errorCount":0,"warningCount":0}]);
    assert_eq!(lint, healthy_lint);
    assert_eq!(cross_file, healthy_lint);
}

#[test]
fn client_only_apps_keep_non_ssr_boundary_findings() {
    let directory = tempfile::tempdir().unwrap();
    project(
        directory.path(),
        "export default defineNuxtConfig({ssr:false})",
        false,
    );
    write(
        directory.path(),
        "src/App.vue",
        "<script setup>\nwindow.innerWidth;\n</script><template><div>{{ JSON.parse('{}') }}</div></template>",
    );
    let (doctor, lint, cross_file) = packets(directory.path(), "client-only-error-boundary");
    assert_eq!(
        doctor
            .findings()
            .iter()
            .map(|finding| finding.code.as_str())
            .collect::<Vec<_>>(),
        ["VIZE_DOCTOR_CF_UNCAUGHT_ERROR"]
    );
    assert_eq!(rule_ids(&lint), Vec::<&str>::new());
    assert_eq!(rule_ids(&cross_file), ["cross-file"]);
}

#[test]
fn commands_launched_in_a_source_directory_keep_the_app_context() {
    let directory = tempfile::tempdir().unwrap();
    project(
        directory.path(),
        "export default defineNuxtConfig({ssr:false})",
        false,
    );
    let source_root = directory.path().join("src");
    let doctor = run(
        &source_root,
        "nested-cwd.doctor",
        &["doctor", ".", "--format", "json", "--exit-zero"],
    );
    assert_eq!(
        doctor,
        serde_json::to_value(DoctorReport::new(".", [])).unwrap()
    );
    let lint = run(
        &source_root,
        "nested-cwd.cross-file",
        &["lint", "App.vue", "--cross-file", "--format", "json"],
    );
    assert_eq!(
        lint,
        serde_json::json!([{"file":"App.vue","messages":[],"errorCount":0,"warningCount":0}])
    );
}

fn project(root: &Path, nuxt: &str, vite_client_build: bool) {
    write(root, "src/App.vue", BROWSER_SOURCE);
    let build = if vite_client_build {
        "build:{ssr:false},"
    } else {
        ""
    };
    let compiler = if vite_client_build {
        "compiler:{ssr:false},"
    } else {
        ""
    };
    write(
        root,
        "vite.config.mjs",
        &format!(
            "export default {{{build}vue:{{{compiler}linter:{{preset:'incremental',rules:{{'ssr/no-browser-globals-in-ssr':'warn'}}}}}}}}"
        ),
    );
    if !nuxt.is_empty() {
        write(root, "nuxt.config.ts", nuxt);
    }
}

fn packets(root: &Path, name: &str) -> (DoctorReport, serde_json::Value, serde_json::Value) {
    let doctor = run(
        root,
        &format!("{name}.doctor"),
        &["doctor", "src", "--format", "json", "--exit-zero"],
    );
    let lint = run(
        root,
        &format!("{name}.lint"),
        &["lint", "src/App.vue", "--format", "json"],
    );
    let cross = run(
        root,
        &format!("{name}.cross-file"),
        &["lint", "src/App.vue", "--cross-file", "--format", "json"],
    );
    (serde_json::from_value(doctor).unwrap(), lint, cross)
}

fn run(root: &Path, name: &str, arguments: &[&str]) -> serde_json::Value {
    let start = Instant::now();
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(arguments)
        .output()
        .unwrap();
    if let Some(artifacts) = std::env::var_os("VIZE_SSR_CONTEXT_ARTIFACTS") {
        let artifacts = Path::new(&artifacts);
        fs::create_dir_all(artifacts).unwrap();
        fs::write(artifacts.join(format!("{name}.json")), &output.stdout).unwrap();
        fs::write(artifacts.join(format!("{name}.stderr")), &output.stderr).unwrap();
        fs::write(artifacts.join(format!("{name}.measurement.json")), serde_json::to_vec_pretty(&serde_json::json!({"elapsedMicros":start.elapsed().as_micros(),"exitCode":output.status.code()})).unwrap()).unwrap();
    }
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn rule_ids(report: &serde_json::Value) -> Vec<&str> {
    report
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| file.get("messages").unwrap().as_array().unwrap())
        .map(|message| message.get("ruleId").unwrap().as_str().unwrap())
        .collect()
}

fn write(root: &Path, relative: &str, source: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}
