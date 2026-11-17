use crate::support::fix_history_runner::check_pack;

#[test]
fn generic_component_aliases_are_instantiated_for_consumers() {
    check_pack(
        "generic-module-alias-instantiation-3365",
        "fix_history_cases::generic_module_alias_instantiation_3365::generic_component_aliases_are_instantiated_for_consumers",
        "a40aefa70fc004f00c230efa9e9303b52f0c10a2",
        Some(3365),
        &[
            "generic-aliases-instantiated",
            "generic-emit-payload-mismatch",
            "generic-slot-prop-missing-member",
        ],
    );
}
