use crate::support::fix_history_runner::check_pack;

#[test]
fn component_use_template_ref_uses_define_expose_surface() {
    check_pack(
        "component-template-ref-expose-4489",
        "fix_history_cases::component_template_ref_expose_4489::component_use_template_ref_uses_define_expose_surface",
        "16f992cb64e502db4066cda3003ec5bad0a0b04d",
        Some(4489),
        &[
            "exposed-member-through-use-template-ref",
            "unexposed-member-through-use-template-ref",
            "native-refs-named-like-setup-bindings",
            "native-canvas-ref-wrong-member",
        ],
    );
}
