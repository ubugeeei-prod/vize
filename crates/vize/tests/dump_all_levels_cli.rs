//! The CLI and playground consume the same native stage ladder capture.

#![expect(clippy::disallowed_macros, reason = "tests assert JSON and CLI bytes")]

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use davinci_test_support::schema;
use vize_curator::inspector::{StageFeed, ladder_run, spolvero_value_with_remarks};

fn run(path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .args(["dump", "--all-levels", "--json"])
        .arg(path)
        .output()
        .unwrap()
}

#[test]
fn raw_template_exports_the_same_pages_and_remarks_as_the_stage_view() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("template.html");
    let template = "<div :class=\"cls\">{{ msg }}</div>";
    fs::write(&path, template).unwrap();

    let output = run(&path);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty());

    let capture = ladder_run(path.to_str().unwrap(), template, &|| 0);
    // `analyzeSfc` uses this same constructor for its stage feed. Only the
    // producer command differs between the WASM result and this CLI output.
    let mut wasm_feed = spolvero_value_with_remarks(
        "analyze-sfc",
        capture.pages.clone(),
        capture.remarks.clone(),
    );
    wasm_feed["command"] = "vize-dump".into();
    let expected = StageFeed {
        command: "vize-dump".into(),
        pages: capture.pages,
        remarks: capture.remarks,
    };
    assert_eq!(output.stdout, expected.to_json().as_bytes());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json, wasm_feed);
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("docs/davinci/plan/spolvero-feed.schema.json");
    let schema_json: serde_json::Value =
        serde_json::from_slice(&fs::read(schema_path).unwrap()).unwrap();
    assert_eq!(schema::validate(&schema_json, &json, "$"), Ok(()));
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["pages"][0]["stage"], "s1");
    assert_eq!(json["pages"][0]["text"], template);
    assert!(
        json["pages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|page| page["stage"] == "s2")
    );
    assert!(
        json["pages"]
            .as_array()
            .unwrap()
            .iter()
            .any(|page| page["stage"] == "s3")
    );
    assert!(!json["remarks"].as_array().unwrap().is_empty());
}

#[test]
fn raw_template_is_an_input_without_an_sfc_wrapper() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("template.html");
    fs::write(&path, "<span>日本語</span>\n").unwrap();
    let output = run(&path);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["pages"][0]["text"], "<span>日本語</span>\n");
    assert_eq!(json["pages"][0]["path"], path.to_str().unwrap());
}

#[test]
fn sfc_input_requires_the_native_container() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("App.vue");
    fs::write(&path, "<template><p>hi</p></template>").unwrap();
    let output = run(&path);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        format!(
            "dump: {}: Vue SFC capture awaits the native L1 container; pass a raw template file\n",
            path.display()
        ),
    );
}

#[test]
fn pug_is_not_silently_treated_as_html() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("template.pug");
    fs::write(&path, "div Hello").unwrap();
    let output = run(&path);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        format!(
            "dump: {}: Pug stage capture is not wired to the native ladder yet\n",
            path.display()
        ),
    );
}
