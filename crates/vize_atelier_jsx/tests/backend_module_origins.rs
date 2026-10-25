//! Caller-built shared roots retain their full authored-module origins.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "test packets use ordinary owned strings and JSON fixtures"
)]

use oxc_sourcemap::SourceMap;
use serde_json::{Value, json};
use vize_atelier_core::{lane::transform_with_source_text, options::TransformOptions};
use vize_atelier_jsx::{JsxLang, lower_source};
use vize_atelier_ssr::{
    SsrCompilerExperimentalOptions, SsrCompilerOptions, compile_l2_to_ssr,
    compile_l2_to_ssr_with_experimental_options,
};
use vize_atelier_vapor::{
    VaporGenerateExperimentalOptions, VaporGenerateOptions, generate_vapor_from_transformed_root,
    generate_vapor_with_options, transform_to_ir,
};
use vize_l0::Allocator;

fn byte_at(text: &str, line: u32, column: u32) -> usize {
    assert!(line <= text.bytes().filter(|&byte| byte == b'\n').count() as u32);
    let start = text
        .split_inclusive('\n')
        .take(line as usize)
        .map(str::len)
        .sum::<usize>();
    let mut units = 0;
    for (offset, ch) in text.get(start..).unwrap().char_indices() {
        if units == column {
            return start + offset;
        }
        assert_ne!(ch, '\n', "source map column exceeds line extent");
        units += ch.len_utf16() as u32;
        assert!(units <= column, "source map splits a UTF-16 surrogate pair");
    }
    assert_eq!(units, column);
    text.len()
}

fn check_map(code: &str, source: &str, raw: &str) -> Value {
    let document: Value = serde_json::from_str(raw).unwrap();
    assert_eq!(document["version"], 3);
    assert_eq!(document["sources"], json!(["Original 日本語.tsx"]));
    assert_eq!(document["sourcesContent"], json!([source]));
    let map = SourceMap::from_json_string(raw).unwrap();
    let tokens = map
        .get_source_view_tokens()
        .map(|token| {
            let generated = byte_at(code, token.get_dst_line(), token.get_dst_col());
            let authored = byte_at(source, token.get_src_line(), token.get_src_col());
            assert!(code.is_char_boundary(generated));
            assert!(source.is_char_boundary(authored));
            json!({ "generated": generated, "authored": authored, "name": token.get_name() })
        })
        .collect::<Vec<_>>();
    let label = source.rfind("label").unwrap();
    assert!(tokens.iter().any(|token| token["authored"] == label));
    assert!(
        tokens.len() > 1,
        "a synthetic module-start anchor cannot qualify"
    );
    json!({ "code": code, "map": document, "tokens": tokens })
}

#[test]
fn caller_built_jsx_roots_keep_real_vapor_and_ssr_origins() {
    let mut packets = Vec::new();
    for (prefix, ending) in [
        ("// before\n", "\n"),
        ("// 日本語 😀 'quoted'\r\n", "\r\n"),
        ("// edit adds an earlier line\n// 日本語 😀\n", "\n"),
    ] {
        let source = format!(
            "{prefix}const label: string = 'text';{ending}export const App = () => <p>{{label}}</p>;{ending}"
        );
        let allocator = Allocator::new();
        let mut lowered = lower_source(&allocator, allocator.as_oxc(), &source, JsxLang::Tsx);
        assert!(lowered.diagnostics.is_empty());
        assert_eq!(lowered.roots.len(), 1);
        let root = lowered.roots.pop().unwrap();
        let analysis = allocator.alloc_owned(lowered.analysis);
        let mut transformed = root.root;
        transform_with_source_text(
            &allocator,
            &mut transformed,
            TransformOptions {
                prefix_identifiers: false,
                vapor: true,
                ..Default::default()
            },
            Some(analysis),
            &source,
        );
        let legacy_ir = transform_to_ir(&allocator, &transformed, &source);
        let legacy = generate_vapor_with_options(
            &legacy_ir,
            None,
            VaporGenerateOptions { jsx_closure: true },
        );
        for mapped in [false, true] {
            let (output, errors, warnings) = generate_vapor_from_transformed_root(
                &allocator,
                &transformed,
                &source,
                None,
                None,
                VaporGenerateOptions { jsx_closure: true },
                VaporGenerateExperimentalOptions {
                    source_map: mapped,
                    source_map_filename: Some("Original 日本語.tsx"),
                    ..Default::default()
                },
            );
            assert!(errors.is_empty(), "{errors:?}");
            assert!(warnings.is_empty(), "{warnings:?}");
            assert_eq!(output.code, legacy.code);
            assert_eq!(output.templates, legacy.templates);
            assert_eq!(output.map.is_some(), mapped);
            if let Some(raw) = &output.map {
                packets.push(json!({ "backend": "vapor", "source": source, "output": check_map(&output.code, &source, raw) }));
            }
        }

        let l2 = root.l2.unwrap();
        let options = SsrCompilerOptions::default();
        let legacy = compile_l2_to_ssr(&allocator, &source, &l2.root, &options).unwrap();
        for mapped in [false, true] {
            let output = compile_l2_to_ssr_with_experimental_options(
                &allocator,
                &source,
                &l2.root,
                &options,
                &SsrCompilerExperimentalOptions {
                    source_map: mapped,
                    source_map_filename: Some("Original 日本語.tsx".into()),
                    ..Default::default()
                },
            )
            .expect("the real L2 plan must own this JSX root");
            assert_eq!(output.code, legacy.code);
            assert_eq!(output.preamble, legacy.preamble);
            assert_eq!(output.map.is_some(), mapped);
            if let Some(raw) = &output.map {
                packets.push(json!({ "backend": "ssr", "source": source, "output": check_map(&output.code, &source, raw), "preamble": output.preamble }));
            }
        }
    }
    assert_eq!(packets.len(), 6);
    if let Some(path) = std::env::var_os("VIZE_JSX_BACKEND_ORIGIN_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&packets).unwrap()).unwrap();
    }
}
