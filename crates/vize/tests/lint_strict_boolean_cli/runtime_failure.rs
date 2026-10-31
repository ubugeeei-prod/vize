//! A requested type-aware check cannot silently succeed without its runtime.
use super::{RULE, vite_config};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

const SOURCE: &str = include_str!(
    "../../../../tests/_fixtures/differential/linter/type-aware-runtime-initialization-11147/App.vue.txt"
);
const ORIGINAL: &str = include_str!(
    "../../../../tests/_fixtures/differential/linter/native-boolean-tsrs-11147/App.vue.txt"
);
const CONFIG: &str = include_str!(
    "../../../../tests/_fixtures/differential/linter/type-aware-runtime-initialization-11147/config.json"
);
const HELP: &str = "Requested type-aware lint could not initialize its native runtime. Check the reported failure, install a compatible native TypeScript runtime, or configure typeChecker.corsaPath, CORSA_PATH, or TSGO_PATH to its executable.";

fn write_config(root: &Path, config: &Value) {
    fs::write(
        root.join("vite.config.mjs"),
        vite_config::source(serde_json::to_vec(config).expect("config JSON")),
    )
    .expect("Vite config");
}

fn project() -> tempfile::TempDir {
    assert_eq!(
        SOURCE, ORIGINAL,
        "the original authored Card.vue is unchanged"
    );
    let root = tempfile::tempdir().expect("project");
    fs::write(root.path().join("App.vue"), SOURCE).expect("original source");
    fs::write(root.path().join("package.json"), r#"{"type":"module"}"#).expect("package");
    fs::write(root.path().join("tsconfig.json"), r#"{"compilerOptions":{"strict":true,"module":"esnext","moduleResolution":"bundler","target":"esnext","jsx":"preserve","skipLibCheck":true,"noEmit":true},"include":["*.vue"]}"#).expect("original compiler options");
    root
}

fn lint(root: &Path, flags: &[&str]) -> (Option<i32>, Value) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command.current_dir(root).args(["lint", "--format", "json"]);
    command.args(flags).arg("App.vue");
    for name in vize_carton::corsa_resolver::CORSA_ENV_VARS {
        command.env_remove(name);
    }
    let output = command.output().expect("real CLI process");
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
    let actual = serde_json::from_slice(&output.stdout).expect("complete JSON report");
    assert_eq!(
        fs::read_to_string(root.join("App.vue")).expect("source"),
        SOURCE
    );
    (output.status.code(), actual)
}

fn runtime_error(root: &Path) -> Value {
    let missing = root
        .canonicalize()
        .expect("project root")
        .join("missing-native-runtime");
    assert!(!missing.exists());
    json!([{
        "file": "App.vue", "errorCount": 1, "warningCount": 0,
        "messages": [{
            "ruleId": "type/corsa-runtime",
            "ruleDocsPath": "docs/content/rules/type-and-script.md",
            "severity": 2,
            "message": format!("[vize:type/corsa-runtime] Configured Corsa executable does not exist: {} (from configuration)", missing.display()),
            "line": 1, "column": 1, "endLine": 1, "endColumn": 1,
            "help": HELP
        }]
    }])
}

#[test]
fn requested_type_aware_initialization_failure_is_a_cli_error_without_source_edits() {
    let root = project();
    let config: Value = serde_json::from_str(CONFIG).expect("pinned config");
    write_config(root.path(), &config);
    for flags in [vec![], vec!["--type-aware"], vec!["--type-aware", "--fix"]] {
        assert_eq!(
            lint(root.path(), &flags),
            (Some(1), runtime_error(root.path()))
        );
    }
}

#[test]
fn explicit_cli_opt_in_cannot_succeed_when_its_runtime_is_missing() {
    let root = project();
    let mut config: Value = serde_json::from_str(CONFIG).expect("pinned config");
    config["linter"]["typeAware"] = json!(false);
    write_config(root.path(), &config);
    assert_eq!(
        lint(root.path(), &["--type-aware"]),
        (Some(1), runtime_error(root.path()))
    );
}

#[test]
fn disabled_or_query_free_type_rules_do_not_require_a_runtime() {
    let root = project();
    let mut config: Value = serde_json::from_str(CONFIG).expect("pinned config");
    config["linter"]["rules"][RULE] = json!("off");
    for static_rule in [false, true] {
        if static_rule {
            config["linter"]["rules"]["type/require-typed-props"] = json!("error");
        }
        write_config(root.path(), &config);
        assert_eq!(
            lint(root.path(), &["--type-aware"]),
            (
                Some(0),
                json!([{
                    "file": "App.vue", "messages": [], "errorCount": 0, "warningCount": 0
                }])
            )
        );
    }
}
