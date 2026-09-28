use vize_atelier_sfc::{SfcParseOptions, compile_sfc, parse_sfc, types::SfcCompileOptions};

#[test]
fn scoped_style_renames_keyframes_and_animation() {
    let source = r#"<template><span class="spin" /></template>
<style scoped>
.spin { animation: spin 1s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
</style>"#;
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    let result = compile_sfc(&descriptor, SfcCompileOptions::default()).unwrap();
    insta::assert_snapshot!(result.css.unwrap());
}
