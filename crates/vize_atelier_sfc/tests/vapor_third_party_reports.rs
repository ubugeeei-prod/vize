//! Original Hebilicious reports retain whole-SFC runtime differential coverage.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "regressions retain complete original sources and runtime packets"
)]

mod third_party_vapor;
use third_party_vapor as report;

#[test]
fn reported_svg_branches_and_dynamic_class_style_match_locked_vue() {
    report::compare(
        include_str!("../../../tests/_fixtures/differential/compiler/vapor-svg-8624/App.vue.txt"),
        include_str!("../../../tests/_fixtures/differential/compiler/vapor-svg-8624/Child.vue.txt"),
        include_str!("../../../tests/_fixtures/differential/compiler/vapor-svg-8624/controls.json"),
    );
}

#[test]
fn reported_renamed_and_nested_slot_props_match_locked_vue() {
    report::compare(
        include_str!(
            "../../../tests/_fixtures/differential/compiler/vapor-slot-destructure-8626/App.vue.txt"
        ),
        &serde_json::from_str::<String>(include_str!(
            "../../../tests/_fixtures/differential/compiler/vapor-slot-destructure-8626/Child.vue.json"
        ))
        .expect("whole original SlotOwner source"),
        include_str!(
            "../../../tests/_fixtures/differential/compiler/vapor-slot-destructure-8626/controls.json"
        ),
    );
}
