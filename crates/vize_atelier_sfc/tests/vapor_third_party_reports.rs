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
