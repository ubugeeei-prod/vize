#![cfg(test)]

use crate::support::{corsa_path, corsa_requirement};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

mod capture;

const SOURCE: &str = r#"type Required = { readonly kind: "expected"; readonly nested: { readonly label: "🙂"; readonly enabled: true } };
const value: Required = { kind: "actual", nested: { label: "違う", enabled: false } };
import Missing from "./Missing.vue";
export { value, Missing };
"#;

const MESSAGES: [&str; 4] = [
    "Type '\"actual\"' is not assignable to type '\"expected\"'.",
    "Type '\"違う\"' is not assignable to type '\"🙂\"'.",
    "Type 'false' is not assignable to type 'true'.",
    "Cannot find module './Missing.vue' or its corresponding type declarations.",
];

fn expected_report(diagnostics: Value) -> Value {
    let errors = diagnostics.as_array().unwrap().len();
    json!({
        "files": [{"file":"src/entry.ts", "diagnostics":diagnostics}],
        "programs": [{"root":".", "tsconfig":"tsconfig.json", "compilerOptions":{
            "module":"ESNext", "moduleResolution":"bundler", "noEmit":true,
            "noErrorTruncation":true, "strict":true, "target":"ES2022"
        }, "files":["src/entry.ts"]}],
        "errorCount":errors, "warningCount":0, "fileCount":1
    })
}

fn counter(profile: &Value, key: &str, samples: u64) -> u64 {
    let row = profile["counters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["key"] == key)
        .unwrap_or_else(|| panic!("missing counter {key}"));
    assert_eq!(row["samples"], samples, "{key}");
    row["total"].as_u64().unwrap()
}

fn assert_parse_profile(profile: &Value, entries: u64, message_bytes: u64) {
    assert_eq!(profile["schema_version"], 1);
    assert_eq!(
        profile["truncation"],
        json!({"dropped_spans":0,"dropped_counters":0})
    );
    for key in [
        "canon.corsa.cli.parse.decode",
        "canon.corsa.cli.parse.assemble",
        "canon.corsa.cli.parse.restore_paths",
        "canon.corsa.cli.parse.deduplicate",
    ] {
        let span = profile["spans"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["key"] == key)
            .unwrap();
        assert_eq!(span["count"], 1, "{key}");
    }
    for key in [
        "canon.corsa.cli.decoded.entries",
        "canon.corsa.cli.assembled.entries",
        "canon.corsa.cli.restored.entries",
        "canon.corsa.cli.deduplicated.entries",
    ] {
        assert_eq!(counter(profile, key, 1), entries, "{key}");
    }
    for key in [
        "canon.corsa.cli.assembled.message_bytes",
        "canon.corsa.cli.restored.message_bytes",
        "canon.corsa.cli.deduplicated.message_bytes",
    ] {
        assert_eq!(counter(profile, key, 1), message_bytes, "{key}");
    }
    assert_eq!(counter(profile, "canon.corsa.cli.stderr.bytes", 1), 0);
}

#[test]
fn real_check_message_profile_preserves_complete_native_packets_and_repair() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let Some(runtime) = corsa_requirement::required_or_skip(corsa_path::resolve(workspace)) else {
        return;
    };
    let project = tempfile::tempdir().unwrap();
    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(
        project.path().join("package.json"),
        r#"{"private":true,"type":"module"}"#,
    )
    .unwrap();
    fs::write(project.path().join("tsconfig.json"), r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true,"noErrorTruncation":true},"include":["src"]}"#).unwrap();
    fs::write(project.path().join("src/entry.ts"), SOURCE).unwrap();
    let run = |profile| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
        command
            .current_dir(project.path())
            .env("CORSA_PATH", &runtime)
            .args(["check", "--no-config", "--format", "json", "."]);
        if profile {
            command.args(["--profile-json", "profile.json"]);
        }
        command.output().unwrap()
    };
    let plain = run(false);
    capture::retain(project.path(), &runtime, "plain", &plain, false);
    assert_eq!(plain.status.code(), Some(1));
    let profiled = run(true);
    capture::retain(project.path(), &runtime, "profiled", &profiled, true);
    assert_eq!(profiled.status.code(), plain.status.code());
    assert_eq!(
        profiled.stdout, plain.stdout,
        "every complete packet byte must survive profiling"
    );
    assert_eq!(profiled.stderr, plain.stderr);
    assert_eq!(
        serde_json::from_slice::<Value>(&profiled.stdout).unwrap(),
        expected_report(json!([
            "error:2:27 [TS2322] Type '\"actual\"' is not assignable to type '\"expected\"'.",
            "error:2:53 [TS2322] Type '\"違う\"' is not assignable to type '\"🙂\"'.",
            "error:2:66 [TS2322] Type 'false' is not assignable to type 'true'.",
            "error:3:21 [TS2307] Cannot find module './Missing.vue' or its corresponding type declarations."
        ]))
    );
    let profile: Value =
        serde_json::from_slice(&fs::read(project.path().join("profile.json")).unwrap()).unwrap();
    let message_bytes = MESSAGES
        .iter()
        .map(|message| message.len() as u64)
        .sum::<u64>();
    assert_parse_profile(&profile, 4, message_bytes);
    // Projection assembly and final authored-message restoration each inspect
    // all four messages; only the first missing-import pass rewrites a mirror.
    assert_eq!(
        counter(&profile, "canon.diagnostics.restore_specifiers.calls", 8),
        8
    );
    assert_eq!(
        counter(
            &profile,
            "canon.diagnostics.restore_specifiers.quoted_candidates",
            8
        ),
        14
    );
    assert_eq!(
        counter(
            &profile,
            "canon.diagnostics.restore_specifiers.eligible_candidates",
            8
        ),
        1
    );
    assert_eq!(
        counter(
            &profile,
            "canon.diagnostics.restore_specifiers.source_searches",
            8
        ),
        14
    );
    assert_eq!(
        counter(
            &profile,
            "canon.diagnostics.restore_specifiers.source_search_input_bytes",
            8
        ),
        14 * SOURCE.len() as u64
    );
    assert_eq!(
        counter(&profile, "canon.diagnostics.restore_specifiers.rewrites", 8),
        1
    );
    // Complete raw 7.0.2 output has four lines (400 UTF-8 bytes), including
    // the generated missing-import spelling that the mapper must restore.
    assert_eq!(counter(&profile, "canon.corsa.cli.stdout.bytes", 1), 400);
    assert_eq!(
        counter(&profile, "canon.corsa.cli.decoded.message_bytes", 1),
        260
    );
    assert_eq!(
        counter(
            &profile,
            "canon.diagnostics.restore_specifiers.message_input_bytes",
            8
        ),
        260 + message_bytes
    );
    fs::write(
        project.path().join("src/entry.ts"),
        "export const fixed: number = 1;\n",
    )
    .unwrap();
    let repaired = run(true);
    capture::retain(project.path(), &runtime, "repaired", &repaired, true);
    assert_eq!(repaired.status.code(), Some(0));
    assert_eq!(
        serde_json::from_slice::<Value>(&repaired.stdout).unwrap(),
        expected_report(json!([]))
    );
    let repaired_profile: Value =
        serde_json::from_slice(&fs::read(project.path().join("profile.json")).unwrap()).unwrap();
    assert_parse_profile(&repaired_profile, 0, 0);
    assert_eq!(
        counter(&repaired_profile, "canon.corsa.cli.stdout.bytes", 1),
        0
    );
    assert_eq!(
        counter(
            &repaired_profile,
            "canon.corsa.cli.decoded.message_bytes",
            1
        ),
        0
    );
    assert!(
        !repaired_profile["counters"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["key"] == "canon.diagnostics.restore_specifiers.calls")
    );
}
