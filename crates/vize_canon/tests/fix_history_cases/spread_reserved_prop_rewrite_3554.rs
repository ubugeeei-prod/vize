use crate::support::fix_history_runner::check_pack;

#[test]
fn v_bind_spreads_rewrite_reserved_prop_references() {
    check_pack(
        "spread-reserved-prop-rewrite-3554",
        "fix_history_cases::spread_reserved_prop_rewrite_3554::v_bind_spreads_rewrite_reserved_prop_references",
        "2342d1fc46ca3a9a927228d8686ce25153332aea",
        Some(3554),
        &[
            "spread-object-reserved-template-props",
            "spread-reserved-rewrite-valid-member",
            "spread-reserved-rewrite-following-missing-member",
            "v-for-reserved-local-spread-number",
            "v-for-reserved-local-spread-string",
        ],
    );
}
