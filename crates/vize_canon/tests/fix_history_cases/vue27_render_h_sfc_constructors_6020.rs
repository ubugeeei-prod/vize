use crate::support::fix_history_runner::check_pack;

#[test]
fn vue27_render_h_accepts_generated_sfc_constructors() {
    check_pack(
        "vue27-render-h-sfc-constructors-6020",
        "fix_history_cases::vue27_render_h_sfc_constructors_6020::vue27_render_h_accepts_generated_sfc_constructors",
        "9193b13ca62af15076609a3a8d7e2405b5df98d1",
        Some(6020),
        &[
            "imported-sfc-in-render-h",
            "imported-generic-sfc-in-render-h",
            "imported-sfc-static-wrong-member",
        ],
    );
}
