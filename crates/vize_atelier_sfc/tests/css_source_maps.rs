//! Source maps belong to the actual parsed CSS, never an untracked prepass.
#![cfg(feature = "native")]

use vize_atelier_sfc::{CssCompileOptions, compile_css};

#[test]
fn complete_unicode_crlf_rule_maps_have_independent_utf16_origins() {
    let source = "\r\n.雪😀 { color: red }\r\n.b{opacity:0.5}";
    let filename = "/fixture/组件 space'\"&😀.css";
    for (minify, expected_code, mappings) in [
        (
            false,
            ".雪😀 {\n  color: red;\n}\n\n.b {\n  opacity: .5;\n}\n",
            "AACA;;;;AACA",
        ),
        (true, ".雪😀{color:red}.b{opacity:.5}", "AACA,eACA"),
    ] {
        let options = CssCompileOptions {
            filename: Some(filename.into()),
            minify,
            ..Default::default()
        };
        let plain = compile_css(source, &options);
        let mapped = compile_css(
            source,
            &CssCompileOptions {
                source_map: true,
                ..options
            },
        );
        assert_eq!(mapped.code, expected_code);
        let map = mapped
            .map
            .as_deref()
            .expect("actual input rules have native origins");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(map).unwrap(),
            serde_json::json!({
                "version": 3, "file": filename, "sources": [filename], "sourcesContent": [source], "names": [], "mappings": mappings
            })
        );
        let mut unmapped = serde_json::to_value(mapped).unwrap();
        unmapped.as_object_mut().unwrap().remove("map");
        assert_eq!(unmapped, serde_json::to_value(plain).unwrap());
    }
}

#[test]
fn both_same_line_columns_use_utf16_with_astral_selectors_and_values() {
    let source = ".雪😀{content:'雪😀'}.b{opacity:.5}";
    let result = compile_css(
        source,
        &CssCompileOptions {
            minify: true,
            source_map: true,
            ..Default::default()
        },
    );
    assert_eq!(result.code, ".雪😀{content:\"雪😀\"}.b{opacity:.5}");
    let map = oxc_sourcemap::SourceMap::from_json_string(result.map.as_deref().unwrap()).unwrap();
    let tokens: Vec<_> = map
        .get_tokens()
        .map(|token| {
            (
                token.get_dst_line(),
                token.get_dst_col(),
                token.get_src_line(),
                token.get_src_col(),
            )
        })
        .collect();
    assert_eq!(tokens, [(0, 0, 0, 0), (0, 19, 0, 19)]);
}

#[test]
fn modules_keyframes_custom_properties_assets_and_targets_preserve_every_non_map_field() {
    for source in [
        ".雪😀{--theme:blue;color:var(--theme)}.b{background:url('../a space😀.png')}",
        "@keyframes spin{from{opacity:0}to{opacity:1}}.box{animation:spin 1s}",
        "@import './雪😀.css';.a{user-select:none;display:flex}",
    ] {
        for css_modules in [false, true] {
            for minify in [false, true] {
                let options = CssCompileOptions {
                    filename: Some("/src/雪😀.css".into()),
                    css_modules,
                    minify,
                    targets: Some(vize_atelier_sfc::CssTargets {
                        safari: Some(12),
                        ..Default::default()
                    }),
                    ..Default::default()
                };
                let plain = compile_css(source, &options);
                let mapped = compile_css(
                    source,
                    &CssCompileOptions {
                        source_map: true,
                        ..options
                    },
                );
                assert!(mapped.errors.is_empty(), "{:?}", mapped.errors);
                assert!(mapped.map.is_some());
                let mut actual = serde_json::to_value(mapped).unwrap();
                actual.as_object_mut().unwrap().remove("map");
                assert_eq!(actual, serde_json::to_value(plain).unwrap());
            }
        }
    }
}

#[test]
fn transformed_prepasses_do_not_acquire_false_input_coordinates() {
    for source in [
        ".a{color:v-bind(theme)}.b{color:blue}",
        ".a{color:red}.b{color:blue}",
        ".host{& .child{color:red}}",
        ":slotted(.a){color:red}",
    ] {
        let options = CssCompileOptions {
            scoped: true,
            scope_id: Some("data-v-123abc".into()),
            ..Default::default()
        };
        let plain = compile_css(source, &options);
        let mapped = compile_css(
            source,
            &CssCompileOptions {
                source_map: true,
                ..options
            },
        );
        assert_eq!(
            serde_json::to_value(mapped).unwrap(),
            serde_json::to_value(plain).unwrap()
        );
    }
}

#[test]
fn empty_error_and_embedded_map_boundaries_do_not_fabricate_origins() {
    for source in [
        "",
        "/* comment */",
        "}",
        "a{opacity:sign(-50%)}",
        ".a{color:red}/*# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VzIjpbIm90aGVyLnNjc3MiXSwic291cmNlc0NvbnRlbnQiOlsiLnNvdXJjZXtjb2xvcjpyZWR9Il0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBIn0= */",
    ] {
        let plain = compile_css(source, &CssCompileOptions::default());
        let mapped = compile_css(
            source,
            &CssCompileOptions {
                source_map: true,
                ..Default::default()
            },
        );
        assert_eq!(
            serde_json::to_value(mapped).unwrap(),
            serde_json::to_value(plain).unwrap()
        );
    }
}

#[test]
fn parser_recovery_keeps_its_existing_result_and_real_rule_origin() {
    let source = ".a{color:";
    let plain = compile_css(source, &CssCompileOptions::default());
    let mapped = compile_css(
        source,
        &CssCompileOptions {
            source_map: true,
            ..Default::default()
        },
    );
    assert_eq!(mapped.code, ".a {\n  color: ;\n}\n");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(mapped.map.as_deref().unwrap()).unwrap(),
        serde_json::json!({
            "version": 3, "file": "style.css", "sources": ["style.css"], "sourcesContent": [source], "names": [], "mappings": "AAAA"
        })
    );
    let mut preserved = serde_json::to_value(mapped).unwrap();
    preserved.as_object_mut().unwrap().remove("map");
    assert_eq!(preserved, serde_json::to_value(plain).unwrap());
}
