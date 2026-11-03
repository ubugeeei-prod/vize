//! Remark document and canonical page contracts through the typed substrate.

#![expect(clippy::expect_used, reason = "committed schemas are test oracles")]

use std::path::Path;

use davinci_test_support::schema as schema_check;
use vize_l0::dump::remarks::RemarkLog;
use vize_l0::dump::{Dump, Mode as DumpMode};

fn load_schema() -> serde_json::Value {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/davinci/plan/remarks.schema.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("schema reads"))
        .expect("schema is valid JSON")
}

#[test]
fn an_empty_remark_log_has_a_canonical_schema_valid_document() {
    let text = RemarkLog::default().to_json("typed-pass-test");
    assert_eq!(
        text.as_str(),
        "{\"schema_version\":1,\"command\":\"typed-pass-test\",\"remarks\":[]}\n"
    );
    let json = serde_json::from_str(text.as_str()).expect("valid JSON");
    assert_eq!(schema_check::validate(&load_schema(), &json, "$"), Ok(()));
}

#[test]
fn the_remarks_page_roundtrips_byte_identically() {
    let page = "[remarks]\n\n[remarks.entries]\n\
                s2.hoist-static applied static-subtree @27:59 tag=\"h1\"\n\
                s2.hoist-static missed static-props @60:99 tag=\"p\" blocker=\"binding\" op=\"ui.on\"\n\n";
    let parsed = RemarkLog::parse(page).expect("committed page parses");
    assert_eq!(parsed.print_to_string(DumpMode::Full).as_str(), page);
}
