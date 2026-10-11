//! Whole SVG bindings retain inherited namespace through reactive updates.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "complete authored namespace differential regression"
)]

mod third_party_vapor;

#[test]
fn svg_and_html_integration_point_bindings_match_locked_vdom() {
    third_party_vapor::compare(
        include_str!(
            "../../../tests/_fixtures/differential/compiler/vapor-svg-namespace-8624/App.vue.txt"
        ),
        include_str!(
            "../../../tests/_fixtures/differential/compiler/vapor-svg-namespace-8624/Child.vue.txt"
        ),
        include_str!(
            "../../../tests/_fixtures/differential/compiler/vapor-svg-namespace-8624/controls.json"
        ),
    );
}
