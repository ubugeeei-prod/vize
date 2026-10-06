//! Original four-rule command, complete JSON reports, and retained raw process outcomes.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};
use vize_carton::{String, append, cstr};

const FIXTURE: &str = "../../../tests/_fixtures/differential/linter/css-value-tokens-7989/";
const CASES: &str =
    include_str!("../../../tests/_fixtures/differential/linter/css-value-tokens-7989/cases.json");
const CONFIG: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/css-value-tokens-7989/vize.config.json.txt"
);
const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/css-value-tokens-7989/MyNotes.vue.txt"
);
const ARGV: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/css-value-tokens-7989/original-command.txt"
);

fn sha(bytes: &[u8]) -> String {
    let mut output = String::default();
    for byte in Sha256::digest(bytes) {
        append!(output, "{byte:02x}");
    }
    output
}

fn invoke(root: &Path, args: &[&str], id: &str, capture: &Path) -> std::process::Output {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    fs::write(capture.join(cstr!("{id}.stdout").as_str()), &output.stdout).unwrap();
    fs::write(capture.join(cstr!("{id}.stderr").as_str()), &output.stderr).unwrap();
    fs::write(
        capture.join(cstr!("{id}.process.json").as_str()),
        serde_json::to_vec_pretty(&json!({
            "case": id, "binary": env!("CARGO_BIN_EXE_vize"), "argv": args,
            "status": output.status.code(), "success": output.status.success(),
            "stdoutSha256": sha(&output.stdout), "stderrSha256": sha(&output.stderr)
        }))
        .unwrap(),
    )
    .unwrap();
    output
}

#[test]
fn original_command_and_all_css_vectors_keep_whole_public_reports() {
    let corpus: Value = serde_json::from_str(CASES).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 17);
    assert_eq!(cases[0]["source"], ORIGINAL);
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let provenance: Value =
        serde_json::from_str(&fs::read_to_string(fixture.join("source.json")).unwrap()).unwrap();
    for (name, pin) in provenance["originalCarriers"].as_object().unwrap() {
        let bytes = fs::read(fixture.join(name)).unwrap();
        assert_eq!(Some(bytes.len() as u64), pin["bytes"].as_u64());
        assert_eq!(sha(&bytes), pin["sha256"].as_str().unwrap());
    }
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    fs::write(root.join("vize.config.json"), CONFIG).unwrap();
    let profile = std::env::var("NEXTEST_PROFILE").unwrap_or_else(|_| "full".into());
    let captures = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/nextest")
        .join(profile)
        .join("css-value-tokens-7989-cli");
    fs::create_dir_all(&captures).unwrap();
    let capture = tempfile::Builder::new()
        .prefix("invocation-")
        .tempdir_in(captures)
        .unwrap()
        .keep();
    fs::write(root.join("MyNotes.vue"), ORIGINAL).unwrap();
    let args: Vec<_> = ARGV.split_whitespace().skip(1).collect();
    assert_eq!(
        args,
        ["lint", "-f", "plain", "--help-level", "none", "MyNotes.vue"]
    );
    let original = invoke(root, &args, "original-command", &capture);
    let mut failures = Vec::new();
    if original.status.code() != Some(0)
        || !original.stderr.is_empty()
        || original.stdout != cases[0]["plain"].as_str().unwrap().as_bytes()
    {
        failures.push(
            json!({"case": "original-command", "status": original.status.code(),
            "stdout": original.stdout, "stderr": original.stderr, "expected": cases[0]["plain"]}),
        );
    }
    assert_eq!(
        fs::read(root.join("MyNotes.vue")).unwrap(),
        ORIGINAL.as_bytes()
    );
    fs::remove_file(root.join("MyNotes.vue")).unwrap();
    for case in cases {
        let filename = case["filename"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        assert_eq!(sha(source.as_bytes()), case["sha256"].as_str().unwrap());
        fs::write(root.join(filename), source).unwrap();
        let output = invoke(
            root,
            &["lint", "--format", "json", filename],
            case["id"].as_str().unwrap(),
            &capture,
        );
        let actual = serde_json::from_slice::<Value>(&output.stdout).ok();
        if output.status.code() != Some(0)
            || !output.stderr.is_empty()
            || actual.as_ref() != Some(&case["cli"])
        {
            failures.push(json!({"case": case["id"], "status": output.status.code(),
                "actual": actual, "expected": case["cli"], "stderr": output.stderr}));
        }
        assert_eq!(fs::read(root.join(filename)).unwrap(), source.as_bytes());
        assert_eq!(
            fs::read(root.join("vize.config.json")).unwrap(),
            CONFIG.as_bytes()
        );
        fs::remove_file(root.join(filename)).unwrap();
    }
    fs::write(
        capture.join("whole-failures.json"),
        serde_json::to_vec_pretty(&failures).unwrap(),
    )
    .unwrap();
    assert_eq!(
        failures,
        Vec::<Value>::new(),
        "all whole reports; raw at {capture:?}"
    );
}
