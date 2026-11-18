use crate::support::fix_history_runner::check_pack;

#[test]
fn forwarded_optional_component_prop_keeps_undefined() {
    check_pack(
        "forwarded-optional-prop-undefined-897",
        "fix_history_cases::forwarded_optional_prop_undefined_897::forwarded_optional_component_prop_keeps_undefined",
        "b2442feba555cc28ca656f4aa32b9f8e015bf6ec",
        Some(897),
        &[
            "forwarded-optional-prop-accepted",
            "forwarded-wider-optional-prop-rejected",
        ],
    );
}
