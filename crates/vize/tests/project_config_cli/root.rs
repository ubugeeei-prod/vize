//! The public CLI follows Vite's selected project without changing explicit inputs.

use super::support::{assert_no_dedicated_config, assert_success, run, write};
use serde_json::Value;
use std::{fs, path::Path};

const SETTINGS: &str = include_str!(
    "../../../../tests/_fixtures/differential/config/vite-root-8371/vite.config.mjs.txt"
);
const APP: &str =
    include_str!("../../../../tests/_fixtures/differential/config/vite-root-8371/App.vue.txt");
const IMAGE: &str =
    include_str!("../../../../tests/_fixtures/differential/config/vite-root-8371/Image.vue.txt");
const TSCONFIG: &str = include_str!(
    "../../../../tests/_fixtures/differential/config/vite-root-8371/tsconfig.json.txt"
);

fn project() -> tempfile::TempDir {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    for (name, source) in [
        ("package.json", "{\"type\":\"module\"}"),
        ("vite.config.mjs", SETTINGS),
        ("app/src/App.vue", APP),
        ("app/src/Image.vue", IMAGE),
        ("app/src/Ignored.vue", IMAGE),
        ("src/Decoy.vue", IMAGE),
        ("app/tsconfig.json", TSCONFIG),
        ("tsconfig.json", TSCONFIG),
        ("app/src/valid.ts", "export const count: number = 1;\n"),
        ("src/invalid.ts", "export const count: number = 'wrong';\n"),
    ] {
        write(root, name, source);
    }
    project
}

fn reset_evaluations(root: &Path) {
    let _ = fs::remove_file(root.join(".evaluations"));
}

fn assert_evaluated_once(root: &Path) {
    assert_eq!(
        fs::read_to_string(root.join(".evaluations")).unwrap(),
        "evaluated\n"
    );
    assert_no_dedicated_config(root);
}

#[test]
fn omitted_build_inputs_use_vite_root_and_explicit_inputs_remain_local() {
    let project = project();
    let root = project.path();
    assert_success(&run(root, &["build", "-o", "configured"]));
    let compiled = fs::read_to_string(root.join("configured/src/App.js")).unwrap();
    assert!(compiled.contains("  two  spaces  "), "{compiled}");
    assert!(!compiled.contains("resolveComponent"), "{compiled}");
    assert!(!root.join("configured/src/Decoy.js").exists());
    assert_evaluated_once(root);

    reset_evaluations(root);
    assert_success(&run(
        root,
        &[
            "build",
            "src/Decoy.vue",
            "-o",
            "explicit",
            "--config",
            "vite.config.mjs",
        ],
    ));
    assert!(root.join("explicit/Decoy.js").exists());
    assert!(!root.join("explicit/App.js").exists());
    assert_evaluated_once(root);

    reset_evaluations(root);
    assert_success(&run(root, &["build", "--no-config", "-o", "defaults"]));
    assert!(root.join("defaults/src/Decoy.js").exists());
    assert!(!root.join(".evaluations").exists());
}

fn lint_report(root: &Path, args: &[&str]) -> (std::process::Output, Value) {
    let mut command = vec!["lint", "--format", "json"];
    command.extend_from_slice(args);
    let output = run(root, &command);
    let report = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {output:?}"));
    (output, report)
}

#[test]
fn omitted_lint_inputs_scope_entry_rules_and_ignores_to_vite_root() {
    let project = project();
    let root = project.path();
    let (output, report) = lint_report(root, &[]);
    assert_eq!(output.status.code(), Some(1), "{output:?}\n{report:#}");
    let files = report.as_array().unwrap();
    assert!(
        files.iter().any(|file| file["file"]
            .as_str()
            .is_some_and(|path| path.ends_with("Image.vue"))
            && file["messages"]
                .as_array()
                .unwrap()
                .iter()
                .any(|message| message["ruleId"] == "a11y/alt-text" && message["severity"] == 2)),
        "{report:#}"
    );
    assert!(
        files.iter().all(
            |file| !file["file"].as_str().unwrap().ends_with("Decoy.vue")
                && !file["file"].as_str().unwrap().ends_with("Ignored.vue")
        ),
        "{report:#}"
    );
    assert_evaluated_once(root);

    reset_evaluations(root);
    let (explicit, report) = lint_report(root, &["src/Decoy.vue", "--config", "vite.config.mjs"]);
    assert_success(&explicit);
    assert!(
        report[0]["file"].as_str().unwrap().ends_with("Decoy.vue"),
        "{report:#}"
    );
    assert!(
        report[0]["messages"]
            .as_array()
            .unwrap()
            .iter()
            .all(|message| message["ruleId"] != "a11y/alt-text"),
        "{report:#}"
    );
    assert_evaluated_once(root);

    reset_evaluations(root);
    let (_, defaults) = lint_report(root, &["--no-config"]);
    assert!(
        defaults
            .as_array()
            .unwrap()
            .iter()
            .any(|file| file["file"].as_str().unwrap().ends_with("Decoy.vue")),
        "{defaults:#}"
    );
    assert!(!root.join(".evaluations").exists());
}

#[cfg(feature = "glyph")]
#[test]
fn omitted_format_inputs_follow_vite_root_without_writing_siblings() {
    let project = project();
    let root = project.path();
    write(root, "src/Decoy.vue", APP);
    assert_success(&run(root, &["fmt", "--write"]));
    let formatted = fs::read_to_string(root.join("app/src/App.vue")).unwrap();
    assert!(formatted.contains("const message = 'hello'"), "{formatted}");
    assert_eq!(fs::read_to_string(root.join("src/Decoy.vue")).unwrap(), APP);
    assert_eq!(
        fs::read_to_string(root.join("app/src/Ignored.vue")).unwrap(),
        IMAGE
    );
    assert_eq!(
        fs::read_to_string(root.join("vite.config.mjs")).unwrap(),
        SETTINGS
    );
    assert_evaluated_once(root);

    reset_evaluations(root);
    assert_success(&run(
        root,
        &[
            "fmt",
            "--write",
            "src/Decoy.vue",
            "--config",
            "vite.config.mjs",
        ],
    ));
    assert!(
        fs::read_to_string(root.join("src/Decoy.vue"))
            .unwrap()
            .contains("'hello'")
    );
    assert_evaluated_once(root);

    reset_evaluations(root);
    let missing = run(root, &["fmt", "--write", "src/Image.vue"]);
    assert_eq!(missing.status.code(), Some(1), "{missing:?}");
    assert_evaluated_once(root);

    reset_evaluations(root);
    write(root, "src/Decoy.vue", APP);
    assert_success(&run(
        root,
        &["fmt", "--write", "src/Decoy.vue", "--no-config"],
    ));
    assert!(
        fs::read_to_string(root.join("src/Decoy.vue"))
            .unwrap()
            .contains("\"hello\"")
    );
    assert!(!root.join(".evaluations").exists());
}

#[test]
fn native_vite_arrays_keep_global_settings_and_root_relative_scoped_entries() {
    let project = project();
    let root = project.path();
    write(
        root,
        "vite.config.mjs",
        "export default {root:'app',vize:[{linter:{preset:'essential',rules:{'a11y/alt-text':'off'}}},{files:['src/**/*.vue'],ignores:['src/Ignored.vue'],linter:{rules:{'a11y/alt-text':'error'}}}]};",
    );
    let (output, report) = lint_report(root, &[]);
    assert_eq!(output.status.code(), Some(1), "{output:?}\n{report:#}");
    assert!(
        report
            .as_array()
            .unwrap()
            .iter()
            .all(|file| !file["file"].as_str().unwrap().ends_with("Decoy.vue")),
        "{report:#}"
    );
    assert!(
        report.as_array().unwrap().iter().any(|file| file["file"]
            .as_str()
            .unwrap()
            .ends_with("Image.vue")
            && file["messages"]
                .as_array()
                .unwrap()
                .iter()
                .any(|message| message["ruleId"] == "a11y/alt-text")),
        "{report:#}"
    );
    assert_no_dedicated_config(root);
}

#[test]
fn dedicated_configuration_keeps_the_existing_invocation_root() {
    let project = project();
    let root = project.path();
    write(
        root,
        "vize.config.json",
        "{\"linter\":{\"preset\":\"essential\",\"rules\":{\"a11y/alt-text\":\"off\"}}}",
    );
    assert_success(&run(root, &["build", "-o", "dedicated"]));
    assert!(root.join("dedicated/src/Decoy.js").exists());
    let app = fs::read_to_string(root.join("dedicated/app/src/App.js")).unwrap();
    assert!(app.contains("resolveComponent"), "{app}");
    let (_, report) = lint_report(root, &[]);
    assert!(
        report
            .as_array()
            .unwrap()
            .iter()
            .any(|file| file["file"].as_str().unwrap().ends_with("Decoy.vue")),
        "{report:#}"
    );
    assert!(!root.join(".evaluations").exists());
}

#[test]
fn omitted_check_inputs_use_vite_tsconfig_and_explicit_tsconfig_stays_local() {
    use super::{corsa_path, corsa_requirement, typecheck};
    use std::process::Command;

    let Some(corsa) =
        corsa_requirement::required_or_skip(corsa_path::resolve(typecheck::workspace_root()))
    else {
        return;
    };
    let project = project();
    let root = project.path();
    typecheck::link_vue(root);
    let check = |args: &[&str]| {
        reset_evaluations(root);
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(root)
            .env("CORSA_PATH", &corsa)
            .env("RAYON_NUM_THREADS", "1")
            .args(["check", "--format", "json"])
            .args(args)
            .output()
            .unwrap();
        let report: Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|error| panic!("{error}: {output:?}"));
        (output, report)
    };
    let (output, report) = check(&[]);
    assert_success(&output);
    assert_eq!(report["errorCount"], 0, "{report:#}");
    assert!(
        report["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|file| file["file"].as_str().unwrap().ends_with("valid.ts")),
        "{report:#}"
    );
    assert!(
        report["files"]
            .as_array()
            .unwrap()
            .iter()
            .all(|file| !file["file"].as_str().unwrap().ends_with("invalid.ts")),
        "{report:#}"
    );
    assert_evaluated_once(root);

    for args in [
        &["--tsconfig", "tsconfig.json"][..],
        &["src/invalid.ts"][..],
        &["--no-config"][..],
    ] {
        let (output, report) = check(args);
        assert_eq!(
            output.status.code(),
            Some(1),
            "{args:?}: {output:?}\n{report:#}"
        );
        assert!(
            report["files"]
                .as_array()
                .unwrap()
                .iter()
                .any(|file| file["file"].as_str().unwrap().ends_with("invalid.ts")),
            "{report:#}"
        );
        if args.contains(&"--no-config") {
            assert!(!root.join(".evaluations").exists());
        } else {
            assert_evaluated_once(root);
        }
    }
    write(
        root,
        "manual.json",
        "{\"typeChecker\":{\"tsconfig\":\"tsconfig.json\"}}",
    );
    let (output, report) = check(&["--config", "manual.json"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}\n{report:#}");
    assert!(!root.join(".evaluations").exists());
}
