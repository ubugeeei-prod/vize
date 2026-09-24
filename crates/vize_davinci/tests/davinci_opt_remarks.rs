//! `davinci-opt`'s P3-13 remark surface, pinned end to end: `--remarks`
//! writes the run's remark document (schema-valid, byte-exact), and the
//! `[remarks]` page is a `--roundtrip` stage like any other folio.

#![expect(clippy::expect_used, reason = "tests assert by panicking")]

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use davinci_test_support::schema as schema_check;
use vize_s0::cstr;

const BUDGET: &str =
    "[budget-observer]\nwalks=2\npasses=3\nanalyses=0\npipelines=1\nfailures=0\n\n";

fn run(args: &[&str], stdin: Option<&str>) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_davinci-opt"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("davinci-opt spawns");
    if let Some(text) = stdin {
        child
            .stdin
            .as_mut()
            .expect("stdin is piped")
            .write_all(text.as_bytes())
            .expect("stdin accepts the folio");
    }
    drop(child.stdin.take());
    child.wait_with_output().expect("davinci-opt exits")
}

fn text_of(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes).expect("UTF-8 output")
}

fn temp_path(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("p3-13-remarks");
    std::fs::create_dir_all(&dir).expect("temp dir creates");
    dir.join(name)
}

fn load_schema() -> serde_json::Value {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/davinci/plan/remarks.schema.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("schema reads"))
        .expect("schema is valid JSON")
}

#[test]
fn remarks_writes_the_run_document_even_when_empty() {
    let path = temp_path("run.json");
    let _ = std::fs::remove_file(&path);
    let output = run(
        &[
            "--pipeline",
            "s2(alpha,beta)",
            "--stage",
            "budget-observer",
            "--remarks",
            path.to_str().expect("UTF-8 path"),
        ],
        Some(BUDGET),
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(text_of(&output.stdout), BUDGET);
    assert_eq!(
        text_of(&output.stderr),
        cstr!(
            "davinci-opt: pipeline s2(alpha,beta): walks=1 passes=2\n\
             davinci-opt: remarks {}: 0 remark(s)\n",
            path.display()
        )
        .as_str()
    );
    // The catalogue-free passes explain nothing, and the document says so
    // rather than being absent.
    let text = std::fs::read_to_string(&path).expect("remark document reads");
    assert_eq!(
        text,
        "{\"schema_version\":1,\"command\":\"davinci-opt\",\"remarks\":[]}\n"
    );
    let json: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
    assert_eq!(schema_check::validate(&load_schema(), &json, "$"), Ok(()));
}

#[test]
fn the_remarks_page_is_a_roundtrip_stage() {
    let page = "[remarks]\n\n[remarks.entries]\n\
                s2.hoist-static applied static-subtree @27:59 tag=\"h1\"\n\
                s2.hoist-static missed static-props @60:99 tag=\"p\" blocker=\"binding\" op=\"ui.on\"\n\n";
    let path = temp_path("page.folio");
    std::fs::write(&path, page).expect("page writes");
    let file = path.to_str().expect("UTF-8 path");
    let output = run(&["--roundtrip", file, "--stage", "remarks"], None);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        text_of(&output.stdout),
        cstr!("roundtrip OK: {file} ({} bytes)\n", page.len()).as_str()
    );
    assert_eq!(text_of(&output.stderr), "");
}
