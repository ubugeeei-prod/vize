use vize_atlas::Compilation;
use vize_relief::TemplateSyntaxMode;

use crate::{
    SfcCompileOptions, SfcCompileProduct, SfcCompileRequest, SfcCompileSettings,
    compile_sfc_with_template_syntax, parse_sfc,
};

#[test]
fn descriptor_scoped_inference_matches_preexisting_build_normalization() {
    let source_text = r#"<template><div class="plain scoped" /></template>
<style>.plain { color: red }</style><style scoped>.scoped { color: blue }</style>"#;
    let mut compilation = Compilation::new();
    crate::register_atlas_providers(&mut compilation).unwrap();
    let source = compilation.add_source("Scoped.vue", source_text).unwrap();
    let request =
        SfcCompileRequest::new(SfcCompileOptions::default(), TemplateSyntaxMode::Standard)
            .with_inferred_scoped_from_descriptor();
    let mut settings = SfcCompileSettings::default();
    settings.insert(source, request);
    settings.install(&mut compilation).unwrap();

    let descriptor = parse_sfc(source_text, Default::default()).unwrap();
    let mut baseline_options = SfcCompileOptions::default();
    baseline_options.parse.filename = "Scoped.vue".into();
    let baseline = compile_sfc_with_template_syntax(
        &descriptor,
        baseline_options.clone(),
        TemplateSyntaxMode::Standard,
    )
    .unwrap();
    let mut normalized = baseline_options;
    normalized.template.scoped = true;
    normalized.style.scoped = true;
    let expected =
        compile_sfc_with_template_syntax(&descriptor, normalized, TemplateSyntaxMode::Standard)
            .unwrap();
    let actual = compilation.query::<SfcCompileProduct>(source).unwrap();

    assert_eq!(actual.value().css, expected.css);
    assert_ne!(actual.value().css, baseline.css);
}
