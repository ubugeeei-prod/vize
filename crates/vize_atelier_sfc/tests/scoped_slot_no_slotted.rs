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

#[test]
fn slot_scope_policy_is_local_to_each_parallel_compile() {
    let ordinary = std::thread::spawn(|| {
        compiled("<template><slot /></template><style scoped>.box { color: red }</style>")
    });
    let slotted = std::thread::spawn(|| {
        compiled("<template><slot /></template><style scoped>:slotted(p) { color: red }</style>")
    });
    let compat = std::thread::spawn(|| {
        compiled(
            "<template><div title=\"&amp;\"><slot /></div></template><style scoped>.box { color: red }</style>",
        )
    });

    let ordinary = ordinary.join().unwrap();
    let slotted = slotted.join().unwrap();
    let compat = compat.join().unwrap();
    assert!(ordinary.contains("_renderSlot(_ctx.$slots, \"default\", {}, undefined, true)"));
    assert!(slotted.contains("_renderSlot(_ctx.$slots, \"default\")"));
    assert!(!slotted.contains("undefined, true"));
    assert!(compat.contains("_renderSlot(_ctx.$slots, \"default\", {}, undefined, true)"));
}
