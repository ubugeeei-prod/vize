//! P2-11 Vue 2 pipe-filter witness: `_resolveFilter` assets and
//! `_filter_*` calls, compared byte-for-byte with the shipped DOM lane.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_s0::config::VueVersion;

const V2_LITERAL_BATTERY: &[(&str, &str)] = &[
    (
        "interpolation_single_literal",
        "<div>{{ 1 | double }}</div>",
    ),
    ("interpolation_args_literal", "<div>{{ 1 | add(2) }}</div>"),
    (
        "interpolation_chain_literal",
        "<div>{{ 1 | f | g(2) }}</div>",
    ),
    (
        "interpolation_dash_name_literal",
        "<div>{{ 1 | foo-bar }}</div>",
    ),
    (
        "interpolation_dollar_name_literal",
        "<div>{{ 1 | $cash }}</div>",
    ),
    ("bind_value_literal", r#"<div :id="1 | formatId"></div>"#),
    ("component_slot_default_literal", "<Foo>{{ 1 | cap }}</Foo>"),
    (
        "slot_outlet_prop_literal",
        r#"<slot :value="1 | formatId"></slot>"#,
    ),
    (
        "interpolation_mixed_text_literal",
        "<div>USD {{ 1 | money }}</div>",
    ),
];

const V3_BATTERY: &[(&str, &str)] = &[("v3_bitwise_or", "<div>{{ message | capitalize }}</div>")];

#[test]
fn s2_vue2_literal_filters_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_prefixed_shipped_literals_with_dialect(
        V2_LITERAL_BATTERY,
        VueVersion::V2,
    );
}

#[test]
fn s2_vue3_keeps_pipe_expressions_out_of_filter_mode() {
    support::assert_s2_matches_shipped_with_dialect(V3_BATTERY, VueVersion::V3);
}
