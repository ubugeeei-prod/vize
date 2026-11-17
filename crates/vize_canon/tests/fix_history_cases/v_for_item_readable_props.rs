use crate::support::fix_history_runner::check_pack;

#[test]
fn v_for_item_props_stay_readable_through_kebab_aliases() {
    check_pack(
        "v-for-item-readable-props",
        "fix_history_cases::v_for_item_readable_props::v_for_item_props_stay_readable_through_kebab_aliases",
        "be40baf2077e555a3e1041b47af369ba6f2e06d4",
        None,
        &[
            "v-for-item-props-forwarded-as-kebab",
            "v-for-item-wrong-prop-type",
        ],
    );
}
