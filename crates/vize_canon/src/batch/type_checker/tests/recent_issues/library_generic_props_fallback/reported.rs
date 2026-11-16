//! The reported #13021 project, pinned in the differential pack, through the
//! production batch checker. The pack's `reported-*` cases are executed here,
//! so their pinned inputs and complete diagnostic packets stay authenticated.

use super::super::super::{
    create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics,
};
use super::Diagnostics;
use std::path::{Path, PathBuf};
use vize_carton::{String, cstr};

fn pack_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/typechecker/library-generic-props-fallback-13021")
}

fn pinned_case(id: &str) -> serde_json::Value {
    let pack: serde_json::Value =
        serde_json::from_slice(&std::fs::read(pack_root().join("cases.json")).unwrap()).unwrap();
    pack["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == id)
        .unwrap_or_else(|| panic!("the pack pins case {id}"))
        .clone()
}

fn run_case(id: &str) -> (Diagnostics, Diagnostics) {
    let case = pinned_case(id);
    let inputs = case["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|input| {
            let source =
                std::fs::read_to_string(pack_root().join(input["source"].as_str().unwrap()))
                    .unwrap();
            (input["file"].as_str().unwrap(), source)
        })
        .collect::<Vec<_>>();
    let files = inputs
        .iter()
        .map(|(file, source)| (*file, source.as_str()))
        .collect::<Vec<_>>();
    let root = create_project_case(&cstr!("library-generic-fallback-{id}"), &files);
    let actual = snapshot_project_diagnostics(&root)
        .expect("the native checker should return a complete diagnostic packet");
    let _ = std::fs::remove_dir_all(root);
    let mut expected = case["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|diagnostic| {
            assert_eq!(diagnostic["severity"], 1);
            (
                String::from(diagnostic["file"].as_str().unwrap()),
                diagnostic["code"].as_u64().map(|code| code as u32),
                cstr!(
                    "{}:{}:error {}",
                    diagnostic["line"],
                    diagnostic["column"],
                    diagnostic["message"].as_str().unwrap()
                ),
            )
        })
        .collect::<Diagnostics>();
    expected.sort();
    (actual, expected)
}

#[test]
fn reported_project_default_mode_lets_undeclared_attributes_fall_through() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let (actual, expected) = run_case("reported-default-loose");
    assert!(
        actual.iter().all(|(_, code, _)| *code != Some(2353)),
        "no undeclared attribute may be an excess property in loose mode: {actual:#?}",
    );
    assert_eq!(
        actual, expected,
        "the pinned loose packet must match exactly"
    );
}

#[test]
fn reported_project_strict_mode_keeps_every_excess_property_row() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let (actual, expected) = run_case("reported-strict");
    assert_eq!(
        actual, expected,
        "the pinned strict packet must match exactly"
    );
}
