use crate::support::fix_history_runner::check_pack;

#[test]
fn direct_template_prop_reads_keep_define_props_result_used() {
    check_pack(
        "define-props-direct-template-reads-2262",
        "fix_history_cases::define_props_direct_template_reads_2262::direct_template_prop_reads_keep_define_props_result_used",
        "a85f3da23989f8208edb7a01407aebfe19e8264a",
        Some(2262),
        &[
            "direct-template-prop-reads",
            "component-signature-consumes-props",
            "optional-direct-prop-member-access",
        ],
    );
}
