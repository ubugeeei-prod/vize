use super::support::{self, cli_packet, config, persist, write};
use crate::support::vite_config;
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

fn run(root: &Path, filename: &str, selection: &[&str]) -> Value {
    let args = [
        "lint",
        filename,
        "--format",
        "json",
        "--locale",
        "en",
        "--help-level",
        "full",
    ];
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(args)
        .args(selection)
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    json!({
        "args": args, "selection": selection, "status": output.status.code(),
        "stdout": String::from_utf8(output.stdout).unwrap(),
        "stderr": String::from_utf8(output.stderr).unwrap(),
    })
}

#[test]
fn all_original_contexts_reach_discovered_and_explicit_cli_routes() {
    let workspace = tempfile::tempdir().unwrap();
    let cases = support::cases();
    let mut observations = Vec::new();
    for case in &cases {
        let root = workspace.path().join(case["id"].as_str().unwrap());
        let filename = case["filename"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let config = config(case);
        write(&root, filename, source);
        write(
            &root,
            "vite.config.mjs",
            &vite_config::source(config.to_string()),
        );
        for selection in [
            &[][..],
            &["--config", "vite.config.mjs"][..],
            &["--config", "."][..],
        ] {
            for repeat in [1, 2] {
                observations.push(
                    json!({"caseId": case["id"], "source": source, "config": config,
                    "repeat": repeat, "expected": cli_packet(case, filename),
                    "actual": run(&root, filename, selection)}),
                );
            }
        }
        assert_eq!(fs::read_to_string(root.join(filename)).unwrap(), source);
        assert_eq!(
            fs::read_to_string(root.join("vite.config.mjs")).unwrap(),
            vite_config::source(config.to_string())
        );
    }
    let evidence = json!({"schema": "vize.component-context.actual-cli-transport-v1",
        "executable": env!("CARGO_BIN_EXE_vize"), "observations": observations});
    persist("cli-transport.json", &evidence);
    assert_eq!(observations.len(), 180);
    for observation in &observations {
        let actual = &observation["actual"];
        assert_eq!(actual["status"], 0, "{observation:#}");
        assert_eq!(actual["stderr"], "", "{observation:#}");
        let packet: Value = serde_json::from_str(actual["stdout"].as_str().unwrap()).unwrap();
        assert_eq!(packet, observation["expected"], "{observation:#}");
    }
}

#[test]
fn compiler_and_lint_policy_share_one_actual_javascript_evaluation() {
    let workspace = tempfile::tempdir().unwrap();
    let root = workspace.path();
    let source = "<template>界😀<my-owned/><my-neighbor/></template>\r\n";
    write(root, "Evaluation.vue", source);
    let settings = r#"import {readFileSync,writeFileSync,existsSync} from 'node:fs';
const file = new URL('./evaluations.txt', import.meta.url);
const previous = existsSync(file) ? Number(readFileSync(file, 'utf8')) : 0;
writeFileSync(file, String(previous + 1));
export default { vue: {
  compiler: {customElements: ['my-owned']},
  linter: {preset: 'incremental', rules: {'vue/require-component-registration': 'warn'}}
} };
"#;
    write(root, "vite.config.mjs", settings);
    let start = source.find("my-neighbor").unwrap();
    let case = json!({"source": source, "authoredExpectedPacket": {
        "diagnostics": [{"message": "Component is used but not explicitly imported",
            "start": start, "end": start + 11,
            "help": "Import the component in <script setup> or register it in components option"}],
        "error_count": 0, "warning_count": 1}});
    let mut observations = Vec::new();
    for selection in [
        &[][..],
        &["--config", "vite.config.mjs"][..],
        &["--config", "."][..],
    ] {
        for _ in [1, 2] {
            let actual = run(root, "Evaluation.vue", selection);
            observations.push(json!({"actual": actual,
                "evaluations": fs::read_to_string(root.join("evaluations.txt")).unwrap()}));
        }
    }
    persist(
        "cli-one-evaluation.json",
        &json!({"source": source, "settings": settings,
        "expected": cli_packet(&case, "Evaluation.vue"), "observations": observations}),
    );
    for (index, observation) in observations.iter().enumerate() {
        assert_eq!(observation["evaluations"], (index + 1).to_string());
        assert_eq!(observation["actual"]["status"], 0, "{observation:#}");
        assert_eq!(observation["actual"]["stderr"], "", "{observation:#}");
        let packet: Value =
            serde_json::from_str(observation["actual"]["stdout"].as_str().unwrap()).unwrap();
        assert_eq!(
            packet,
            cli_packet(&case, "Evaluation.vue"),
            "{observation:#}"
        );
    }
    assert_eq!(
        fs::read_to_string(root.join("Evaluation.vue")).unwrap(),
        source
    );
    assert_eq!(
        fs::read_to_string(root.join("vite.config.mjs")).unwrap(),
        settings
    );
}
