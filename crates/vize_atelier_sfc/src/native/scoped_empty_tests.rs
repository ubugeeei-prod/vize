use super::scoped_tests::verify_css_map;
use super::{NativeSfcCompileError, NativeSfcCompileOptions, compile_native_sfc};
use crate::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
use vize_l0::Allocator;

#[test]
fn three_original_empty_sfc_modules_css_maps_and_scope_id_match_whole_contracts() {
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/native_sfc_scoped_empty_vue_3_5_35.json"
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
        let receipt = syntax.empty_class().unwrap();
        assert!(syntax.simple_class().is_err());
        assert!(core::ptr::eq(receipt.syntax(), syntax));
        assert!(core::ptr::eq(syntax.source().root_source(), source));
        assert!(syntax.simple_class_list().is_err());
        let (colon, pseudo) = receipt.pseudo_tokens();
        assert_eq!(
            &source[colon.span().start as usize..colon.span().end as usize],
            ":"
        );
        assert_eq!(
            &source[pseudo.span().start as usize..pseudo.span().end as usize],
            "empty"
        );
        assert_eq!(
            &source[receipt.pseudo_span().start as usize..receipt.pseudo_span().end as usize],
            ":empty"
        );
        verify_css_map(source, filename, output);
        verify_all_original_bytes_and_generated_scope_gaps(syntax.source().source(), output);
        let map: serde_json::Value = serde_json::from_str(output.source_map().unwrap()).unwrap();
        assert_eq!(map["sourcesContent"], serde_json::json!([source]));
        captures.push(serde_json::json!({ "id":fixture["id"], "source":source, "code":output.code(), "css":output.css(), "scopeId":output.scope_id(), "map":map, "cssMap":serde_json::from_str::<serde_json::Value>(output.css_source_map().unwrap()).unwrap() }));
        let pseudo_span = receipt.pseudo_span();
        let (owner, result) = compilation.into_parts();
        assert_eq!(
            owner.style_syntax()[0].empty_class().unwrap().pseudo_span(),
            pseudo_span
        );
        assert!(core::ptr::eq(
            owner.style_syntax()[0].source().root_source(),
            source
        ));
        assert!(result.is_ok());
    }
    if let Some(path) = std::env::var_os("VIZE_NATIVE_EMPTY_CSS_CAPTURE") {
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
    assert_eq!(gaps, 1);
}

#[test]
fn empty_trim_preserves_every_original_internal_byte_and_one_unlinked_insertion() {
    let source = "<!-- 雪 -->\r\n<style scoped> \r\n.雪:empty \t{ content:'雪🌸'; color:red; } \r\n</style><template><p :class=\"'雪'\"/></template>";
    let css = " \r\n.雪[data-v-trim]:empty \t{ content:'雪🌸'; color:red; } \r\n";
    for trim in [false, true] {
        let arena = Allocator::default();
        let compilation = compile_native_sfc(
            &arena,
            source,
            NativeSfcCompileOptions {
                filename: "Empty雪.vue",
                source_map: true,
                style_trim: trim,
                scope_id: Some("data-v-trim"),
                ..NativeSfcCompileOptions::default()
            },
        );
        let output = compilation.result().unwrap();
        assert_eq!(output.css(), Some(if trim { css.trim() } else { css }));
        verify_css_map(source, "Empty雪.vue", output);
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
fn unproven_empty_syntax_retains_complete_custody_and_no_partial_product() {
    for css in [
        ".a:hover{}",
        ".a:EMPTY{}",
        ".a::empty{}",
        ".a :empty{}",
        ".a: empty{}",
        ".a/**/:empty{}",
        ".a:/**/empty{}",
        ".a:empty/**/{}",
        ".a:empty:empty{}",
        ".a:empty,.b:empty{}",
        ".\\61:empty{}",
        ".a:em\\70 ty{}",
        ".a:empty(){}",
        ".a:empty{color:v-bind(color)}",
        ".a:empty{content:'v-bind(color)'}",
        ".a:empty{content:'v/**/-bind(color)'}",
        ".a:empty{color:red",
        ".a:empty{} .b{}",
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
    for attrs in ["scoped module", "scoped lang='sass'"] {
        let source =
            format!("<template><p/></template><style {attrs}>.a:empty{{color:red}}</style>");
        let arena = Allocator::default();
        let compilation = compile_native_sfc(&arena, &source, NativeSfcCompileOptions::default());
        assert!(matches!(
            compilation.result(),
            Err(NativeSfcCompileError::StyleCompilationUnavailable { .. })
        ));
        let syntax = &compilation.observation().style_syntax()[0];
        assert!(core::ptr::eq(
            syntax.source().root_source(),
            source.as_str()
        ));
        assert_eq!(syntax.source().source(), ".a:empty{color:red}");
    }
    let source = "<template><p/></template><style scoped custom='true'>.a:empty{color:red}</style>";
    let arena = Allocator::default();
    let compilation = compile_native_sfc(&arena, source, NativeSfcCompileOptions::default());
    assert_eq!(
        compilation.result().unwrap_err(),
        NativeSfcCompileError::Descriptor
    );
    let descriptor = compilation.observation().descriptor();
    assert!(
        descriptor.issues().iter().any(|issue| issue.code
            == vize_l1::container::vue::DescriptorIssueCode::UnsupportedAttribute)
    );
    assert!(core::ptr::eq(descriptor.source(), source));
    assert!(compilation.observation().style_syntax().is_empty());
    let block = &descriptor.container().blocks[1];
    assert_eq!(
        &source[block.content.start as usize..block.content.end as usize],
        ".a:empty{color:red}"
    );
}
