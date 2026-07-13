use super::*;

#[test]
fn graph_ssr_emits_the_components_own_scope_on_every_element() {
    let source_text = r#"<script setup>const message = 'scoped'</script>
<template><main><span>{{ message }}</span></main></template>
<style scoped>main { color: red }</style>"#;
    let mut compilation = Compilation::new();
    register_compile_test_providers(&mut compilation);
    let source = compilation
        .add_source("ScopedSsr.vue", source_text)
        .unwrap();
    let mut options = SfcCompileOptions::default();
    options.scope_id = Some("a1b2c3d4".into());
    options.template.ssr = true;
    install_sfc_compile_request(
        &mut compilation,
        source,
        SfcCompileRequest::new(options, TemplateSyntaxMode::Standard),
    )
    .unwrap();

    let compiled = compilation.query::<SfcCompileProduct>(source).unwrap();
    let code = &compiled.value().code;
    assert_eq!(
        code.matches("_push(\" data-v-a1b2c3d4\")").count(),
        2,
        "{code}"
    );
    assert!(code.contains("_sfc_main.__scopeId = \"data-v-a1b2c3d4\""));
}
