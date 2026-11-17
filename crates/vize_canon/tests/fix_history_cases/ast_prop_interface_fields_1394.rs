use crate::support::fix_history_runner::check_pack;

#[test]
fn local_prop_interface_fields_resolve_through_the_ast() {
    check_pack(
        "ast-prop-interface-fields-1394",
        "fix_history_cases::ast_prop_interface_fields_1394::local_prop_interface_fields_resolve_through_the_ast",
        "b76d2f7b3d23c49f55a12f2a7ba41e553304da1f",
        Some(1394),
        &[
            "local-interface-props-in-template",
            "comment-decoy-field-is-not-a-prop",
            "local-interface-props-from-parent",
            "local-interface-generic-field-mismatch",
            "local-intersection-alias-props",
            "local-intersection-alias-wrong-member",
        ],
    );
}
