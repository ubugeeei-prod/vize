use super::css_tests::{decode_map, position};
use super::{NativeSfcCompileError, NativeSfcCompileOptions, compile_native_sfc};
use crate::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
use vize_l0::Allocator;
use vize_l4::module::AssemblyError;

#[test]
fn four_source_bound_scoped_modules_css_and_default_identity_match_complete_contracts() {
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/native_sfc_scoped_css_vue_3_5_35.json"
    ))
    .unwrap();
    let fixtures = pack["fixtures"].as_array().unwrap();
    assert_eq!(fixtures.len(), 4);
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
        assert_eq!(
            output.code(),
            fixture["code"].as_str().unwrap(),
            "{}",
            fixture["id"]
        );
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
        // Ordinary compilation leaves component scope attachment to its Vite
        // consumer; its independently scoped CSS proves the filename identity.
        assert!(
            ordinary
                .css
                .as_ref()
                .unwrap()
                .contains(output.scope_id().unwrap())
        );
        assert!(compilation.observation().admitted().is_some());
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
        for syntax in compilation.observation().style_syntax() {
            assert!(core::ptr::eq(syntax.source().root_source(), source));
            assert!(syntax.simple_class().is_ok());
        }
        verify_css_map(source, filename, output);
        let map: serde_json::Value = serde_json::from_str(output.source_map().unwrap()).unwrap();
        assert_eq!(map["sourcesContent"], serde_json::json!([source]));
        captures.push(serde_json::json!({ "id":fixture["id"], "source":source, "code":output.code(), "css":output.css(), "scopeId":output.scope_id(), "map":map, "cssMap":serde_json::from_str::<serde_json::Value>(output.css_source_map().unwrap()).unwrap() }));
        let moved = Box::new(compilation);
        let (owner, result) = (*moved).into_parts();
        assert_eq!(
            owner.style_syntax().len(),
            fixture["styles"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|style| style["scoped"] == true)
                .count()
        );
        assert!(result.is_ok());
    }
    if let Some(path) = std::env::var_os("VIZE_NATIVE_SCOPED_CSS_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&captures).unwrap()).unwrap();
    }
}

fn verify_css_map(source: &str, filename: &str, output: &super::NativeSfcOutput) {
    let document = output.css_document().unwrap();
    let map: serde_json::Value = serde_json::from_str(output.css_source_map().unwrap()).unwrap();
    assert_eq!(map["sources"], serde_json::json!([filename]));
    assert_eq!(map["sourcesContent"], serde_json::json!([source]));
    let segments = decode_map(map["mappings"].as_str().unwrap());
    assert_eq!(segments.len(), document.links().len());
    for (link, segment) in document.links().iter().zip(segments) {
        assert_eq!(
            &document.as_str()[link.generated.start as usize..link.generated.end as usize],
            &source[link.authored.start as usize..link.authored.end as usize]
        );
        assert_eq!(
            segment,
            (
                position(document.as_str(), link.generated.start),
                position(source, link.authored.start)
            )
        );
        assert!(
            !document.as_str()[link.generated.start as usize..link.generated.end as usize]
                .contains("[data-v-")
        );
    }
}

#[test]
fn escaped_class_and_optional_trim_preserve_every_original_css_byte_and_scope_gap() {
    let source = "<!-- 雪 -->\r\n<style scoped> \r\n.\\66 oo { content:'雪🌸'; color:red; } \r\n</style><template><p :class=\"'foo'\"/></template>";
    for trim in [false, true] {
        let arena = Allocator::default();
        let compilation = compile_native_sfc(
            &arena,
            source,
            NativeSfcCompileOptions {
                filename: "Escaped雪.vue",
                source_map: true,
                style_trim: trim,
                scope_id: Some("data-v-escape"),
                ..NativeSfcCompileOptions::default()
            },
        );
        let output = compilation.result().unwrap();
        let expected = " \r\n.\\66 oo[data-v-escape] { content:'雪🌸'; color:red; } \r\n";
        assert_eq!(
            output.css(),
            Some(if trim { expected.trim() } else { expected })
        );
        verify_css_map(source, "Escaped雪.vue", output);
        let css = output.css_document().unwrap();
        let insertion = css.as_str().find("[data-v-escape]").unwrap() as u32;
        assert!(
            css.links()
                .iter()
                .all(|link| link.generated.end <= insertion
                    || link.generated.start >= insertion + 15)
        );
        let before = compilation.observation().style_syntax()[0]
            .simple_class()
            .unwrap();
        assert_eq!(
            &source[before.insertion() as usize - 6..before.insertion() as usize],
            "\\66 oo"
        );
    }
}

#[test]
fn unsupported_scoped_css_and_invalid_scope_options_never_return_partial_products() {
    for css in [
        "",
        "p{color:red}",
        ".a{} .b{}",
        ". a{}",
        ".a:hover{}",
        ".a{color:v-bind(color)}",
        ".a{color:v\\2d bind(color)}",
        ".a{content:'v-bind(color)'}",
        ".a{content:\"v-bind (color)\"}",
        ".a{content:'v/**/-bind(color)'}",
        ".a{content:'v-/* x */bind(color)'}",
        "@charset 'utf-8';.a{}",
        ".a{color:red",
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
        let syntax = &compilation.observation().style_syntax()[0];
        assert_eq!(container_index, 2);
        assert_eq!(syntax.container_index(), container_index);
        assert!(syntax.source().contains_block_span(issue.span));
        assert!(core::ptr::eq(
            syntax.source().root_source(),
            source.as_str()
        ));
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
        assert!(compilation.observation().admitted().is_some());
        assert!(
            compilation
                .observation()
                .file()
                .unwrap()
                .file()
                .is_complete()
        );
    }
    let source = "<template><p/></template><style scoped>.a{color:red}</style>";
    for scope in ["", "data-v-", "not-prefixed", "data-v-a]", "data-v-a\\62"] {
        let arena = Allocator::default();
        let compilation = compile_native_sfc(
            &arena,
            source,
            NativeSfcCompileOptions {
                scope_id: Some(scope),
                ..NativeSfcCompileOptions::default()
            },
        );
        assert_eq!(
            compilation.result().unwrap_err(),
            NativeSfcCompileError::Assembly(AssemblyError::InvalidScopeId)
        );
        assert!(
            compilation.observation().style_syntax()[0]
                .simple_class()
                .is_ok()
        );
    }
    let arena = Allocator::default();
    let plain = compile_native_sfc(
        &arena,
        "<template><p/></template><style>.a{color:red}</style>",
        NativeSfcCompileOptions::default(),
    );
    assert!(plain.result().unwrap().scope_id().is_none());
    assert!(!plain.result().unwrap().code().contains("__scopeId"));
}
