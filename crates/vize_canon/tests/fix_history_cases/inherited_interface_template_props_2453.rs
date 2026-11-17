use crate::support::fix_history_runner::check_pack;

#[test]
fn local_interface_extends_binds_inherited_template_props() {
    check_pack(
        "inherited-interface-template-props-2453",
        "fix_history_cases::inherited_interface_template_props_2453::local_interface_extends_binds_inherited_template_props",
        "261e88513e2f5411167f7a401a69fcb35b241088",
        Some(2453),
        &[
            "inherited-and-local-props-bound",
            "inherited-prop-wrong-member",
            "local-prop-wrong-member",
        ],
    );
}
