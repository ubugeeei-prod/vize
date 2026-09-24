//! Davinci S2 prop-value source retention witnesses.

#[expect(
    dead_code,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::string_slice,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "shared test support; each binary uses a subset"
)]
mod support;

const BATTERY: &[(&str, &str)] = &[(
    "multiline_conditional_bind_value",
    r#"<Foo size="sm" :title="
          step.status === 'valid' ? t('valid')
          : step.status === 'invalid' ? t('invalid')
          : t('pending')
        " />"#,
)];

#[test]
fn s2_multiline_prop_values_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
