use super::{NativeSfcCompileError, NativeSfcCompileOptions, compile_native_sfc};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{container::vue::DescriptorIssueCode, embed::Lang};
use vize_l4::module::AssemblyError;

#[test]
fn whole_native_sfc_modules_match_complete_pinned_render_references() {
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/native_sfc_dom_vue_3_5_35.json"
    ))
    .unwrap();
    let fixtures = pack
        .get("fixtures")
        .and_then(serde_json::Value::as_array)
        .unwrap();
    assert_eq!(fixtures.len(), 8);
    let mut captures = Vec::new();
    for fixture in fixtures {
        let arena = Allocator::default();
        let source = fixture
            .get("source")
            .and_then(serde_json::Value::as_str)
            .unwrap();
        let expected = fixture
            .get("code")
            .and_then(serde_json::Value::as_str)
            .unwrap();
        let plain = compile_native_sfc(&arena, source, NativeSfcCompileOptions::default());
        let mapped = compile_native_sfc(
            &arena,
            source,
            NativeSfcCompileOptions {
                filename: "Native雪🌸.vue",
                source_map: true,
                ..NativeSfcCompileOptions::default()
            },
        );
        assert!(plain.observation().admitted().is_some());
        assert!(mapped.observation().admitted().is_some());
        let plain_output = plain.result().unwrap();
        let output = mapped.result().unwrap();
        assert_eq!(plain_output.code(), expected);
        assert_eq!(output.code(), expected);
        assert!(plain_output.source_map().is_none());
        assert!(plain_output.document().links().is_empty());
        let map: serde_json::Value = serde_json::from_str(output.source_map().unwrap()).unwrap();
        assert_eq!(&map, fixture.get("nativeMap").unwrap());
        assert_eq!(map.get("version"), Some(&serde_json::json!(3)));
        assert_eq!(map.get("file"), Some(&serde_json::json!("Native雪🌸.vue")));
        assert_eq!(map.get("names"), Some(&serde_json::json!([])));
        assert_eq!(
            map.get("sourcesContent"),
            Some(&serde_json::json!([source]))
        );
        assert_eq!(
            map.get("sources"),
            Some(&serde_json::json!(["Native雪🌸.vue"]))
        );
        assert!(
            !map.get("mappings")
                .and_then(serde_json::Value::as_str)
                .unwrap()
                .is_empty()
        );
        assert!(core::ptr::eq(
            mapped.observation().descriptor().source(),
            source
        ));
        let file = mapped.observation().file().unwrap().file();
        assert!(file.is_complete());
        assert!(core::ptr::eq(file.artifact().source(), source));
        for link in output.document().links() {
            assert!(
                output
                    .code()
                    .get(link.generated.start as usize..link.generated.end as usize)
                    .is_some()
            );
            assert!(
                source
                    .get(link.authored.start as usize..link.authored.end as usize)
                    .is_some()
            );
            assert!(link.authored.start >= 10);
        }
        captures.push(serde_json::json!({
            "id": fixture.get("id"), "source": source, "code": output.code(), "map": map,
        }));
    }
    if let Some(path) = std::env::var_os("VIZE_NATIVE_SFC_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
    }
}

#[test]
fn js_ts_and_empty_script_roles_remain_explicit_product_refusals_with_real_syntax() {
    for (source, lang) in [
        (
            "<script>const value=1</script><template><p/></template>",
            Lang::Js,
        ),
        (
            "<script setup lang=ts>const value=1</script><template><p/></template>",
            Lang::Ts,
        ),
        ("<script setup></script><template><p/></template>", Lang::Js),
        (
            "<script setup>const value=defineProps()</script><template><p/></template>",
            Lang::Js,
        ),
    ] {
        let arena = Allocator::default();
        let compilation = compile_native_sfc(&arena, source, NativeSfcCompileOptions::default());
        let script = compilation.observation().scripts().first().unwrap();
        assert!(
            matches!(compilation.result(), Err(NativeSfcCompileError::ScriptCompilationUnavailable { container_index, span })
            if container_index == script.container_index() && span == script.block().span())
        );
        assert_eq!(script.lang(), lang);
        let syntax = script.syntax().unwrap();
        assert_eq!(syntax.grammar().lang, lang);
        assert_eq!(syntax.source_type().is_typescript(), lang == Lang::Ts);
        assert!(core::ptr::eq(
            syntax.source().text(),
            script.block().source()
        ));
        assert!(core::ptr::eq(script.block().root_source(), source));
    }
}

#[test]
fn external_blocks_are_never_loaded_and_keep_every_original_capture() {
    for source in [
        "<template src='./missing.html'></template>",
        "<script src='./missing.ts'></script><template><p/></template>",
        "<template><p/></template><style src='./missing.css'></style>",
        "<template><p/></template><i18n src='./missing.json'></i18n>",
    ] {
        let arena = Allocator::default();
        let compilation = compile_native_sfc(&arena, source, NativeSfcCompileOptions::default());
        let Err(NativeSfcCompileError::ExternalBlock {
            container_index,
            span,
        }) = compilation.result()
        else {
            panic!("typed external source refusal");
        };
        let Some(block) = compilation
            .observation()
            .descriptor()
            .container()
            .blocks
            .get(container_index)
        else {
            panic!("retained original external block");
        };
        assert_eq!(block.attr("src").unwrap().span, span);
        assert!(core::ptr::eq(
            compilation.observation().descriptor().source(),
            source
        ));
        assert!(compilation.observation().admitted().is_none());
        assert!(compilation.observation().scripts().is_empty());
        assert!(compilation.observation().template().is_none());
    }
}

#[test]
fn style_custom_template_languages_and_jsx_profiles_keep_descriptor_refusals() {
    for (source, issue) in [
        (
            "<template><p/></template><style>p{color:red}</style>",
            DescriptorIssueCode::UnsupportedBlock,
        ),
        (
            "<template><p/></template><custom>original</custom>",
            DescriptorIssueCode::UnsupportedBlock,
        ),
        (
            "<template lang=pug>p hello</template>",
            DescriptorIssueCode::UnsupportedLanguage,
        ),
        (
            "<script lang=jsx>const x=<p/></script><template><p/></template>",
            DescriptorIssueCode::UnsupportedLanguage,
        ),
        (
            "<script setup lang=tsx>const x=<p/></script><template><p/></template>",
            DescriptorIssueCode::UnsupportedLanguage,
        ),
    ] {
        let arena = Allocator::default();
        let compilation = compile_native_sfc(&arena, source, NativeSfcCompileOptions::default());
        assert_eq!(
            compilation.result().unwrap_err(),
            NativeSfcCompileError::Descriptor
        );
        assert!(
            compilation
                .observation()
                .descriptor()
                .issues()
                .iter()
                .any(|actual| actual.code == issue)
        );
        assert!(core::ptr::eq(
            compilation.observation().descriptor().source(),
            source
        ));
        assert!(compilation.observation().scripts().is_empty());
        assert!(compilation.observation().template().is_none());
    }
}

#[test]
fn runtime_and_descriptor_profile_refusals_never_return_partial_modules() {
    let arena = Allocator::default();
    let source = "<template><p/></template>";
    let unsupported_runtime = compile_native_sfc(
        &arena,
        source,
        NativeSfcCompileOptions {
            runtime_version: "3.6.0-rc.9",
            ..NativeSfcCompileOptions::default()
        },
    );
    assert_eq!(
        unsupported_runtime.result().unwrap_err(),
        NativeSfcCompileError::Assembly(AssemblyError::UnsupportedRuntimeVersion)
    );
    assert!(unsupported_runtime.observation().admitted().is_some());
    for (version, dialect, issue) in [
        (
            VueVersion::V2,
            VueDialect::Vue,
            DescriptorIssueCode::UnsupportedVersion,
        ),
        (
            VueVersion::V3,
            VueDialect::PetiteVue,
            DescriptorIssueCode::UnsupportedDialect,
        ),
    ] {
        let mut options = NativeSfcCompileOptions::default();
        options.descriptor.version = version;
        options.descriptor.dialect = dialect;
        let refused = compile_native_sfc(&arena, source, options);
        assert_eq!(
            refused.result().unwrap_err(),
            NativeSfcCompileError::Descriptor
        );
        assert!(
            refused
                .observation()
                .descriptor()
                .issues()
                .iter()
                .any(|actual| actual.code == issue)
        );
    }
    let mut options = NativeSfcCompileOptions::default();
    options.descriptor.template.experimental_in_tag_comments = true;
    let refused = compile_native_sfc(&arena, source, options);
    assert_eq!(
        refused.result().unwrap_err(),
        NativeSfcCompileError::Descriptor
    );
    assert!(
        refused
            .observation()
            .descriptor()
            .issues()
            .iter()
            .any(|actual| actual.code == DescriptorIssueCode::UnsupportedOptions)
    );
}

#[test]
fn malformed_template_refusal_retains_original_diagnostics_and_structural_file() {
    let arena = Allocator::default();
    let source = "<template><p>{{value + /* original */}}</p></template>";
    let compilation = compile_native_sfc(&arena, source, NativeSfcCompileOptions::default());
    assert_eq!(
        compilation.result().unwrap_err(),
        NativeSfcCompileError::Orchestration
    );
    let observed = compilation.observation();
    assert!(observed.admitted().is_none());
    let file = observed.file().unwrap().file();
    assert!(file.is_complete());
    assert!(core::ptr::eq(file.artifact().source(), source));
    let template = observed.template().unwrap();
    assert!(!template.holes().is_empty());
    assert!(!template.diagnostics().is_empty());
    assert!(core::ptr::eq(
        template.component().unwrap().block().root_source(),
        source
    ));
    assert!(!template.embeds().is_empty() || !template.rejected_syntax().is_empty());
}

#[test]
fn ts_template_refusal_keeps_the_genuine_incomplete_file_and_native_owner() {
    let arena = Allocator::default();
    let source =
        "<script setup lang=ts>const value=1</script><template>{{value as number}}</template>";
    let compilation = compile_native_sfc(&arena, source, NativeSfcCompileOptions::default());
    assert!(matches!(
        compilation.result(),
        Err(NativeSfcCompileError::ScriptCompilationUnavailable { .. })
    ));
    let observed = compilation.observation();
    assert!(observed.admitted().is_none());
    let rejected = observed.rejected_file().unwrap();
    let file = rejected.file().unwrap();
    assert!(!file.is_complete());
    assert!(core::ptr::eq(file.artifact().source(), source));
    let template = observed.template().unwrap();
    let produced = template.produced().unwrap();
    assert!(core::ptr::eq(
        produced.component.block().root_source(),
        source
    ));
    assert!(produced.embeds.iter().any(|embed| embed.node.is_none()));
    assert!(!produced.holes.is_empty());
    assert!(!file.template_issues().is_empty());
    assert!(matches!(
        produced.embeds.first().unwrap().syntax.expression(),
        Some(oxc_ast::ast::Expression::TSAsExpression(_))
    ));
}
