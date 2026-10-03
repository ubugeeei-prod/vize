use super::scoped_tests::verify_css_map;
use super::{NativeSfcCompileError, NativeSfcCompileOptions, compile_native_sfc};
use crate::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
use vize_l0::Allocator;

#[test]
fn three_original_class_list_sfc_modules_css_maps_and_scope_id_match_whole_contracts() {
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/native_sfc_scoped_class_list_vue_3_5_35.json"
    ))
    .unwrap();
    let fixtures = pack["fixtures"].as_array().unwrap();
    assert_eq!(fixtures.len(), 3);
    let mut captures = Vec::new();
    for fixture in fixtures {
        let source = fixture["source"].as_str().unwrap();
        let filename = fixture["filename"].as_str().unwrap();
        let explicit = fixture["explicitScopeId"].as_str();
        let arena = Allocator::default();
        let compilation = compile_native_sfc(
            &arena,
            source,
            NativeSfcCompileOptions {
                filename,
                scope_id: explicit,
                source_map: true,
                ..NativeSfcCompileOptions::default()
            },
        );
        let plain = compile_native_sfc(
            &arena,
            source,
            NativeSfcCompileOptions {
                filename,
                scope_id: explicit,
                ..NativeSfcCompileOptions::default()
            },
        );
        let output = compilation.result().expect(fixture["id"].as_str().unwrap());
        assert_eq!(output.code(), fixture["code"].as_str().unwrap());
        assert_eq!(output.css(), fixture["css"].as_str());
        assert_eq!(output.scope_id(), fixture["scopeId"].as_str());
        assert_eq!(output.code(), plain.result().unwrap().code());
        assert_eq!(output.css(), plain.result().unwrap().css());
        assert!(plain.result().unwrap().css_source_map().is_none());
        assert!(
            plain
                .result()
                .unwrap()
                .css_document()
                .unwrap()
                .links()
                .is_empty()
        );
        let descriptor = parse_sfc(
            source,
            SfcParseOptions {
                filename: filename.into(),
                ..SfcParseOptions::default()
            },
        )
        .unwrap();
        let ordinary = compile_sfc(
            &descriptor,
            SfcCompileOptions {
                parse: SfcParseOptions {
                    filename: filename.into(),
                    ..SfcParseOptions::default()
                },
                scope_id: explicit.map(|id| id.strip_prefix("data-v-").unwrap().into()),
                ..SfcCompileOptions::default()
            },
        )
        .unwrap();
        assert_eq!(ordinary.css.as_deref(), fixture["ordinaryCss"].as_str());
        assert!(
            ordinary
                .css
                .as_ref()
                .unwrap()
                .contains(output.scope_id().unwrap())
        );
        assert!(core::ptr::eq(
            compilation
                .observation()
                .file()
                .unwrap()
                .file()
                .artifact()
                .source(),
            source
        ));
        let syntax = &compilation.observation().style_syntax()[0];
        let receipt = syntax.simple_class_list().unwrap();
        assert!(syntax.simple_class().is_err());
        assert!(core::ptr::eq(receipt.syntax(), syntax));
        assert!(core::ptr::eq(syntax.source().root_source(), source));
        assert_eq!(
            receipt.class_count(),
            fixture["classes"].as_array().unwrap().len()
        );
        assert_eq!(receipt.insertions().count(), receipt.class_count());
        verify_css_map(source, filename, output);
        verify_all_original_bytes_and_generated_scope_gaps(syntax.source().source(), output);
        let map: serde_json::Value = serde_json::from_str(output.source_map().unwrap()).unwrap();
        assert_eq!(map["sourcesContent"], serde_json::json!([source]));
        captures.push(serde_json::json!({ "id":fixture["id"], "source":source, "code":output.code(), "css":output.css(), "scopeId":output.scope_id(), "map":map, "cssMap":serde_json::from_str::<serde_json::Value>(output.css_source_map().unwrap()).unwrap() }));
        let (owner, result) = compilation.into_parts();
        assert_eq!(
            owner.style_syntax()[0]
                .simple_class_list()
                .unwrap()
                .class_count(),
            2
        );
        assert!(core::ptr::eq(
            owner.style_syntax()[0].source().root_source(),
            source
        ));
        assert!(result.is_ok());
    }
    if let Some(path) = std::env::var_os("VIZE_NATIVE_CLASS_LIST_CSS_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
    }
}

fn verify_all_original_bytes_and_generated_scope_gaps(
    original: &str,
    output: &super::NativeSfcOutput,
) {
    let document = output.css_document().unwrap();
    let scope = format!("[{}]", output.scope_id().unwrap());
    let mut authored = 0;
    let mut generated = 0;
    let mut gaps = 0;
    for link in document.links() {
        let start = link.generated.start as usize;
        let end = link.generated.end as usize;
        if start > generated {
            assert_eq!(&document.as_str()[generated..start], scope);
            gaps += 1;
        }
        let linked = &document.as_str()[start..end];
        assert_eq!(&original[authored..authored + linked.len()], linked);
        authored += linked.len();
        generated = end;
    }
    assert_eq!(authored, original.len());
    assert_eq!(generated, document.as_str().len());
    assert_eq!(gaps, 2);
}

#[test]
fn class_list_trim_preserves_every_original_internal_byte_and_two_unlinked_insertions() {
    let source = "<!-- 雪 -->\r\n<style scoped> \r\n.雪,\r\n .a { content:'雪🌸'; color:red; } \r\n</style><template><p :class=\"'雪 a'\"/></template>";
    let css = " \r\n.雪[data-v-trim],\r\n .a[data-v-trim] { content:'雪🌸'; color:red; } \r\n";
    for trim in [false, true] {
        let arena = Allocator::default();
        let compilation = compile_native_sfc(
            &arena,
            source,
            NativeSfcCompileOptions {
                filename: "List雪.vue",
                source_map: true,
                style_trim: trim,
                scope_id: Some("data-v-trim"),
                ..NativeSfcCompileOptions::default()
            },
        );
        let output = compilation.result().unwrap();
        assert_eq!(output.css(), Some(if trim { css.trim() } else { css }));
        verify_css_map(source, "List雪.vue", output);
        let original = compilation.observation().style_syntax()[0]
            .source()
            .source();
        verify_all_original_bytes_and_generated_scope_gaps(
            if trim { original.trim() } else { original },
            output,
        );
    }
}

#[test]
fn unproven_list_normalization_or_syntax_retains_complete_custody_and_no_partial_product() {
    for css in [
        ".a , .b{}",
        ".a/* x */,.b{}",
        ".a,/* x */.b{}",
        ".a,.b/* x */{}",
        ".\\61,.b{}",
        ".a,.b:hover{}",
        ".a,p{}",
        ".a,,.b{}",
        ".a,. b{}",
        ".a,.b{color:v-bind(color)}",
        ".a,.b{color:v\\2d bind(color)}",
        ".a,.b{content:'v-bind(color)'}",
        ".a,.b{content:'v/**/-bind(color)'}",
        ".a,.b{content:'v-/* x */bind(color)'}",
        ".a,.b{color:red",
        ".a,.b{} .c{}",
        "@charset 'utf-8';.a,.b{}",
    ] {
        let source = format!(
            "<style>.plain{{color:red}}</style><template><p/></template><style scoped>{css}</style>"
        );
        let arena = Allocator::default();
        let compilation = compile_native_sfc(&arena, &source, NativeSfcCompileOptions::default());
        let Err(NativeSfcCompileError::ScopedStyleUnavailable {
            container_index,
            issue,
        }) = compilation.result()
        else {
            panic!("{css}: {:?}", compilation.result());
        };
        assert_eq!(container_index, 2);
        let syntax = &compilation.observation().style_syntax()[0];
        assert_eq!(syntax.container_index(), container_index);
        assert!(syntax.source().contains_block_span(issue.span));
        assert!(core::ptr::eq(
            syntax.source().root_source(),
            source.as_str()
        ));
        assert_eq!(syntax.source().source(), css);
        assert_eq!(
            compilation
                .observation()
                .descriptor()
                .admitted()
                .unwrap()
                .styles()
                .len(),
            2
        );
        assert!(
            compilation
                .observation()
                .file()
                .unwrap()
                .file()
                .is_complete()
        );
        assert!(compilation.observation().admitted().is_some());
    }
}
