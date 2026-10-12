//! Finite, independently authored successors for archived engine panic inputs.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use vize_atelier_sfc::{CssCompileOptions, compile_css, parse_css_ast, print_css_ast};
use vize_l0::{String, cstr};

const AUTHORITY: &str = include_str!(
    "../../../../tests/_fixtures/differential/css-engine-vendor-exit/finite-successors.json"
);

pub fn authority() -> Value {
    let value: Value = serde_json::from_str(AUTHORITY).expect("authored finite CSS authority");
    assert_eq!(value["schema"], "vize.css-engine-finite-successors");
    assert_eq!(value["atelierRoles"].as_array().unwrap().len(), 14);
    assert_eq!(value["additionalAtelier"].as_array().unwrap().len(), 6);
    value
}

fn assert_source(row: &Value, source: &str) {
    assert_eq!(row["input"]["source"], source, "whole original source");
    let mut digest = String::with_capacity(64);
    for byte in Sha256::digest(source.as_bytes()) {
        digest.push_str(cstr!("{byte:02x}").as_str());
    }
    assert_eq!(
        row["input"]["sourceSha256"].as_str().unwrap(),
        digest.as_str()
    );
}

pub fn assert_whole(row: &Value, source: &str) {
    assert_source(row, source);
    let options = CssCompileOptions::default();
    let parsed = parse_css_ast(source, &options);
    let printed = parsed.ast.clone().map(|ast| print_css_ast(ast, &options));
    let actual = json!({"parsed": parsed, "compiled": compile_css(source, &options),
        "printed": printed});
    assert_eq!(
        actual, row["wholeExpected"],
        "complete AST/error/code/print"
    );
    assert_eq!(
        json!(parse_css_ast(
            "@keyframes x { 50% { opacity: 0.5 } }",
            &options
        )),
        authority()["subsequentHealthy"],
        "complete healthy AST after the original input"
    );
}

pub fn assert_role(law: &str, index: usize, source: &str) {
    let authority = authority();
    let mut matches = authority["atelierRoles"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["input"]["law"] == law && row["input"]["index"] == index);
    let row = matches.next().expect("one named original role");
    assert!(matches.next().is_none(), "no duplicate role authority");
    assert_whole(row, source);
}

pub fn assert_additional(id: &str, source: &str) {
    let authority = authority();
    let mut matches = authority["additionalAtelier"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["input"]["id"] == id);
    let row = matches.next().expect("one named additional original route");
    assert!(
        matches.next().is_none(),
        "no duplicate additional authority"
    );
    assert_whole(row, source);
}
