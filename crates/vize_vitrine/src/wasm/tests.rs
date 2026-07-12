use super::{
    compiler::compile_internal, options::resolve_template_syntax_compat, utf8_byte_to_utf16_offset,
};
use crate::CompilerOptions;
use vize_atelier_core::CodegenOptions;
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, compile_sfc_with_template_syntax_and_codegen_options,
    parse_sfc,
};

#[test]
fn test_utf8_byte_to_utf16_offset_handles_multibyte_and_astral_chars() {
    let source = "aあ😀b";

    let hiragana_start = source.find('あ').expect("hiragana should exist") as u32;
    let emoji_start = source.find('😀').expect("emoji should exist") as u32;
    let latin_b_start = source.find('b').expect("latin b should exist") as u32;

    assert_eq!(utf8_byte_to_utf16_offset(source, 0), 0);
    assert_eq!(utf8_byte_to_utf16_offset(source, hiragana_start), 1);
    assert_eq!(utf8_byte_to_utf16_offset(source, emoji_start), 2);
    assert_eq!(utf8_byte_to_utf16_offset(source, latin_b_start), 4);
    assert_eq!(utf8_byte_to_utf16_offset(source, source.len() as u32), 5);
}

#[test]
fn legacy_vue_parser_quirks_only_falls_back_without_an_explicit_mode() {
    assert_eq!(
        resolve_template_syntax_compat(None, Some(true)).as_deref(),
        Some("quirks")
    );
    assert_eq!(resolve_template_syntax_compat(None, Some(false)), None);
    assert_eq!(
        resolve_template_syntax_compat(Some("strict".to_string()), Some(true)).as_deref(),
        Some("strict")
    );
}

#[test]
fn compiler_options_change_runtime_names_syntax_and_source_maps() {
    let module = compile_internal(
        "<div>{{ message }}</div>",
        &CompilerOptions {
            mode: Some("module".to_string()),
            runtime_module_name: Some("@acme/vue-runtime".to_string()),
            ..Default::default()
        },
        false,
        None,
    )
    .expect("custom runtime module should compile");
    assert!(module.preamble.contains("@acme/vue-runtime"));

    let function = compile_internal(
        "<div />",
        &CompilerOptions {
            runtime_global_name: Some("AcmeVue".to_string()),
            ..Default::default()
        },
        false,
        None,
    )
    .expect("custom runtime global should compile");
    assert!(function.preamble.contains("AcmeVue") || function.code.contains("AcmeVue"));

    let strict = CompilerOptions {
        template_syntax: Some("strict".to_string()),
        ..Default::default()
    };
    assert!(compile_internal("<div /><span></span>", &strict, false, None).is_err());

    let with_map = compile_internal(
        "<div>{{ message }}</div>",
        &CompilerOptions {
            source_map: Some(true),
            filename: Some("src/Component.vue".to_string()),
            ..Default::default()
        },
        false,
        None,
    )
    .expect("source-map compile should succeed");
    assert_eq!(
        with_map
            .map
            .as_ref()
            .and_then(|map| map["version"].as_u64()),
        Some(3)
    );
    assert_eq!(
        with_map
            .map
            .as_ref()
            .and_then(|map| map["sources"][0].as_str()),
        Some("src/Component.vue")
    );
}

#[test]
fn sfc_compiler_options_change_module_and_standalone_runtimes() {
    let descriptor = parse_sfc("<template><div></div></template>", Default::default())
        .expect("fixture should parse");
    let codegen_options = CodegenOptions {
        runtime_module_name: "@acme/vue-runtime".into(),
        runtime_global_name: "AcmeVue".into(),
        ..Default::default()
    };

    for vapor in [false, true] {
        let module = compile_sfc_with_template_syntax_and_codegen_options(
            &descriptor,
            SfcCompileOptions {
                vapor,
                ..Default::default()
            },
            vize_atelier_core::TemplateSyntaxMode::Standard,
            codegen_options.clone(),
        )
        .expect("module SFC should compile");
        assert!(
            module.code.contains("@acme/vue-runtime"),
            "custom module name missing from vapor={vapor}:\n{}",
            module.code
        );

        let standalone = compile_sfc_with_template_syntax_and_codegen_options(
            &descriptor,
            SfcCompileOptions {
                script: ScriptCompileOptions {
                    inline_template: true,
                    ..Default::default()
                },
                vapor,
                ..Default::default()
            },
            vize_atelier_core::TemplateSyntaxMode::Standard,
            codegen_options.clone(),
        )
        .expect("standalone SFC should compile");
        assert!(
            standalone.code.contains("AcmeVue"),
            "custom global name missing from vapor={vapor}:\n{}",
            standalone.code
        );
        assert!(!standalone.code.contains("@acme/vue-runtime"));
    }
}
