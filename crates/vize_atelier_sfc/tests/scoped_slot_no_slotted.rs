#![expect(
    clippy::disallowed_macros,
    reason = "`insta::assert_snapshot!` expands to `format!`"
)]

use vize_atelier_sfc::{SfcCompileOptions, compile_sfc, parse_sfc};

fn compiled(source: &str) -> String {
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    compile_sfc(&descriptor, SfcCompileOptions::default())
        .unwrap()
        .code
        .to_string()
}

#[test]
fn scoped_slot_without_slotted_selector_passes_no_slotted() {
    let code = compiled(
        r#"<template><div><slot /></div></template><style scoped>.box { color: red }</style>"#,
    );
    insta::assert_snapshot!("fast_path", code);

    let compat = compiled(
        r#"<template><div title="&amp;"><slot name="label">fallback</slot></div></template><style scoped>.box { color: red }</style>"#,
    );
    insta::assert_snapshot!("compat_path", compat);
}

#[test]
fn slotted_selector_keeps_runtime_slotted_scope_behavior() {
    let code = compiled(
        r#"<template><slot /></template><style scoped>:slotted(p) { color: red }</style>"#,
    );
    insta::assert_snapshot!("slotted_selector", code);
}
