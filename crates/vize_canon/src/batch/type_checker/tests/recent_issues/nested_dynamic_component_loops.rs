//! Preserve the authored table ancestry before resolving loop bindings (#11728).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_carton::{String, cstr};

const INPUTS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/nested-dynamic-vfor-11728/"
);

#[test]
fn nested_dynamic_component_loops_keep_lexical_values_and_wrong_payloads() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let names = [
        "Clean.vue",
        "Scoped.vue",
        "Errors.vue",
        "Cell.vue",
        "globals.d.ts",
    ];
    let files = names.map(|name| {
        (
            cstr!("src/{name}"),
            std::fs::read_to_string(cstr!("{INPUTS}{name}.txt").as_str()).unwrap(),
        )
    });
    let inputs = files
        .iter()
        .map(|(name, text)| (name.as_str(), text.as_str()))
        .collect::<Vec<_>>();
    let project = super::real_vue::create_project_case("nested-dynamic-vfor-11728", &inputs);
    let diagnostics = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    let reported = diagnostics.map(|rows| {
        rows.into_iter()
            .map(|(file, code, _)| (file, code))
            .collect::<Vec<_>>()
    });
    assert_eq!(
        reported,
        Some(vec![(String::from("src/Errors.vue"), Some(2322))])
    );
}
