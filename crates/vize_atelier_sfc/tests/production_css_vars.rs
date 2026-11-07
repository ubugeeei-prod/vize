use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

#[test]
fn production_css_vars_are_independent_of_checkout_path() {
    let source = r#"<script setup>
const color = "red"
</script>
<template><div class="box">x</div></template>
<style scoped>.box { color: v-bind(color); }</style>"#;
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse");
    let compile = |filename: &str, ssr: bool| {
        let mut options = SfcCompileOptions {
            parse: SfcParseOptions {
                filename: filename.into(),
                ..Default::default()
            },
            scope_id: Some("812be104".into()),
            ..Default::default()
        };
        options.script.id = Some(filename.into());
        options.template.is_prod = true;
        options.template.ssr = ssr;
        compile_sfc(&descriptor, options).expect("compile")
    };

    for ssr in [false, true] {
        let short = compile("/work/a/src/Box.vue", ssr);
        let relocated = compile("/work/very-different-folder/src/Box.vue", ssr);
        assert_eq!(short.css, relocated.css, "CSS changed after relocation");
        assert_eq!(short.code, relocated.code, "JS changed after relocation");
    }
}
