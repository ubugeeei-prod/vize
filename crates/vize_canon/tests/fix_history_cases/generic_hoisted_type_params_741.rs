use crate::support::fix_history_runner::check_pack;

#[test]
fn script_setup_generic_binds_on_hoisted_type_declarations() {
    check_pack(
        "generic-hoisted-type-params-741",
        "fix_history_cases::generic_hoisted_type_params_741::script_setup_generic_binds_on_hoisted_type_declarations",
        "fb8eba1cfeced92863464d715a38744cf52f9607",
        Some(741),
        &[
            "hoisted-alias-references-generic",
            "hoisted-alias-without-space-before-equals",
            "hoisted-interface-references-generic",
            "hoisted-alias-member-type-checked",
        ],
    );
}
