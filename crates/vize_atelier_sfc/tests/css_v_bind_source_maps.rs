#![cfg(feature = "native")]

use vize_atelier_sfc::{CssCompileOptions, compile_css};

#[test]
fn whole_binding_map_points_past_replacement_to_the_actual_authored_rule() {
    let source = ".a{color:v-bind(theme)}.b{color:blue}";
    let filename = "/src/组件雪😀.css";
    let options = CssCompileOptions {
        filename: Some(filename.into()),
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
        mapped.code,
        ".a {\n  color: var(--123abc-theme);\n}\n\n.b {\n  color: #00f;\n}\n"
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(mapped.map.as_deref().unwrap()).unwrap(),
        serde_json::json!({ "version": 3, "file": filename, "names": [], "sources": [filename], "sourcesContent": [source], "mappings": "AAAA;;;;AAAuB" })
    );
    let mut preserved = serde_json::to_value(mapped).unwrap();
    preserved.as_object_mut().unwrap().remove("map");
    assert_eq!(preserved, serde_json::to_value(plain).unwrap());
}

#[test]
fn multiline_unicode_replacements_keep_original_line_and_utf16_rule_columns() {
    let source = ".雪😀{color:v-bind(\r\n '主题😀'\r\n)}.b{opacity:v-bind(next)}\r\n.c{color:red}";
    for minify in [false, true] {
        let options = CssCompileOptions {
            minify,
            source_map: true,
            scope_id: Some("data-v-x".into()),
            ..Default::default()
        };
        let mapped = compile_css(source, &options);
        assert!(mapped.errors.is_empty());
        let map =
            oxc_sourcemap::SourceMap::from_json_string(mapped.map.as_deref().unwrap()).unwrap();
        let origins: Vec<_> = map
            .get_tokens()
            .map(|token| (token.get_src_line(), token.get_src_col()))
            .collect();
        assert_eq!(origins, [(0, 0), (2, 2), (3, 0)]);
        let plain = compile_css(
            source,
            &CssCompileOptions {
                source_map: false,
                ..options
            },
        );
        let mut preserved = serde_json::to_value(mapped).unwrap();
        preserved.as_object_mut().unwrap().remove("map");
        assert_eq!(preserved, serde_json::to_value(plain).unwrap());
    }
}

#[test]
fn scoped_and_embedded_binding_inputs_still_require_their_own_origins() {
    for (source, scoped) in [
        (".a{color:v-bind(theme)}.b{color:blue}", true),
        (
            ".a{color:v-bind(theme)}/*# sourceMappingURL=data:application/json;base64,e30= */",
            false,
        ),
        (".a{color:v-bind(theme)} }", false),
    ] {
        let options = CssCompileOptions {
            scoped,
            scope_id: Some("data-v-x".into()),
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
