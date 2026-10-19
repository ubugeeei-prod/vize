//! Davinci L2 prop-value source retention witnesses.

use crate::support;

const BATTERY: &[(&str, &str)] = &[(
    "multiline_conditional_bind_value",
    r#"<Foo size="sm" :title="
          step.status === 'valid' ? t('valid')
          : step.status === 'invalid' ? t('invalid')
          : t('pending')
        " />"#,
)];

#[test]
fn l2_multiline_prop_values_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_l2_matches_shipped(BATTERY);
}
