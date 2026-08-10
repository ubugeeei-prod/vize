use vize_glyph::{FormatOptions, format_sfc};

#[test]
fn preserves_authored_style_cascade_order() {
    let plain_first = r#"<style>
.shared { color: red; }
</style>

<template><p class="shared">cascade</p></template>

<style scoped>
.shared { color: blue; }
</style>
"#;
    let scoped_first = r#"<style scoped>
.shared { color: blue; }
</style>

<template><p class="shared">cascade</p></template>

<style>
.shared { color: red; }
</style>
"#;
    let options = FormatOptions::default();
    for (source, first_style, second_style) in [
        (plain_first, "<style>", "<style scoped>"),
        (scoped_first, "<style scoped>", "<style>"),
    ] {
        let first = format_sfc(source, &options).unwrap();
        let first_position = first
            .code
            .find(first_style)
            .expect("first authored style block");
        let second_position = first
            .code
            .find(second_style)
            .expect("second authored style block");
        assert!(
            first_position < second_position,
            "formatter reversed the authored CSS cascade"
        );

        let second = format_sfc(&first.code, &options).unwrap();
        assert_eq!(
            first.code, second.code,
            "style order must remain a fixed point"
        );
    }
}
