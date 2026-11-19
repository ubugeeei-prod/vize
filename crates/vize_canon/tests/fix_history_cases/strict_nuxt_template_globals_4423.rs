use crate::support::fix_history_runner::check_pack;

#[test]
fn strict_nuxt_template_globals_read_declared_surfaces() {
    check_pack(
        "strict-nuxt-template-globals-4423",
        "fix_history_cases::strict_nuxt_template_globals_4423::strict_nuxt_template_globals_read_declared_surfaces",
        "d197098cb7a3d4d414a8225dda2a97267b253f82",
        Some(4423),
        &["auto-imports", "unknown-globals", "member-root", "literals"],
    );
}
