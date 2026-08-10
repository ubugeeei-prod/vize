use vize_glyph::{FormatOptions, format_sfc};

#[test]
fn preserves_authored_style_cascade_order() {
    let source = r#"<style>
.shared { color: red; }
</style>

<template><p class="shared">cascade</p></template>

<style scoped>
.shared { color: blue; }
</style>
"#;
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    let plain = first.code.find("<style>").expect("plain style block");
    let scoped = first
        .code
        .find("<style scoped>")
        .expect("scoped style block");
    assert!(
        plain < scoped,
        "formatter reversed the authored CSS cascade"
    );

    let second = format_sfc(&first.code, &options).unwrap();
    assert_eq!(
        first.code, second.code,
        "style order must remain a fixed point"
    );
}
