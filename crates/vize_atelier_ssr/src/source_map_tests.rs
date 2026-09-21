//! Exact span-carrying SSR source maps (Davinci P3-9).
//!
//! Every test pins the full generated code and every decoded segment, rendered
//! as `generated text -> authored text [name]` so each anchor can be reviewed
//! without decoding VLQ by hand. The decoder is local on purpose: it must not
//! share code with the encoder it checks.

#![allow(clippy::disallowed_types, clippy::disallowed_macros)]

use crate::{
    SsrCompilerExperimentalOptions, SsrCompilerOptions, compile_ssr,
    compile_ssr_with_template_syntax_and_experimental_options,
};
use vize_atelier_core::TemplateSyntaxMode;
use vize_s0::Allocator;

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn compile(source: &str, source_map: bool) -> crate::SsrCodegenResult {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_ssr_with_template_syntax_and_experimental_options(
        &allocator,
        source,
        SsrCompilerOptions::default(),
        TemplateSyntaxMode::Standard,
        SsrCompilerExperimentalOptions {
            source_map,
            source_map_filename: Some("Foo.vue".into()),
            ..SsrCompilerExperimentalOptions::default()
        },
    );
    assert_eq!(errors.len(), 0, "Errors: {errors:?}");
    result
}

/// Absolute `(generated line, generated column, source line, source column,
/// name index)` for every segment.
fn decode(mappings: &str) -> std::vec::Vec<(usize, i64, i64, i64, Option<i64>)> {
    let mut segments = std::vec::Vec::new();
    let (mut source_line, mut source_column, mut name) = (0i64, 0i64, 0i64);
    for (line, group) in mappings.split(';').enumerate() {
        let mut column = 0i64;
        for field in group.split(',').filter(|field| !field.is_empty()) {
            let mut values = std::vec::Vec::new();
            let (mut value, mut shift) = (0u64, 0u32);
            for byte in field.bytes() {
                let digit = BASE64.iter().position(|&c| c == byte).unwrap() as u64;
                value |= (digit & 31) << shift;
                shift += 5;
                if digit & 32 == 0 {
                    let magnitude = (value >> 1) as i64;
                    values.push(if value & 1 == 1 {
                        -magnitude
                    } else {
                        magnitude
                    });
                    (value, shift) = (0, 0);
                }
            }
            column += values[0];
            source_line += values[2];
            source_column += values[3];
            let named = values.get(4).map(|delta| {
                name += delta;
                name
            });
            segments.push((line, column, source_line, source_column, named));
        }
    }
    segments
}

fn offset(text: &str, line: usize, column: i64) -> usize {
    let start: usize = text.split_inclusive('\n').take(line).map(str::len).sum();
    start + column as usize
}

fn render(code: &str, source: &str, map: &str) -> std::vec::Vec<std::string::String> {
    let map: serde_json::Value = serde_json::from_str(map).unwrap();
    let names = map["names"].as_array().unwrap();
    let window = |text: &str, at: usize| text[at..(at + 8).min(text.len())].replace('\n', "⏎");
    decode(map["mappings"].as_str().unwrap())
        .into_iter()
        .map(|(line, column, source_line, source_column, name)| {
            let generated = window(code, offset(code, line, column));
            let authored = window(source, offset(source, source_line as usize, source_column));
            let name = name.map_or(std::string::String::new(), |index| {
                std::format!(" [{}]", names[index as usize].as_str().unwrap())
            });
            std::format!("{line}:{column} {generated:?} -> {authored:?}{name}")
        })
        .collect()
}

#[test]
fn source_map_disabled_by_default_yields_none() {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_ssr(&allocator, "<div>{{ msg }}</div>");

    assert!(errors.is_empty(), "Errors: {errors:?}");
    assert!(result.map.is_none());
}

#[test]
fn source_map_is_additive_and_maps_tokens_byte_exactly() {
    let source = "<div class=\"a\"><p :title=\"t\">{{ msg }}</p><b v-if=\"ok\">Y</b><i v-for=\"x in xs\">{{ x }}</i></div>";
    let without_map = compile(source, false);
    let with_map = compile(source, true);
    assert_eq!(
        (with_map.code.as_str(), with_map.preamble.as_str()),
        (without_map.code.as_str(), without_map.preamble.as_str())
    );
    assert_eq!(
        with_map.code.as_str(),
        "function ssrRender(_ctx, _push, _parent, _attrs) {\n  _push(`<div${_ssrRenderAttrs(_mergeProps({ class: \"a\" }, _attrs))}><p${_ssrRenderAttr(\"title\", _ctx.t)}>${_ssrInterpolate(_ctx.msg)}</p>`)\n  if (_ctx.ok) {\n    _push(`<b>Y</b>`)\n  } else {\n    _push(`<!---->`)\n  }\n  _push(`<!--[-->`)\n  _ssrRenderList(_ctx.xs, (x) => {\n    _push(`<i>${_ssrInterpolate(x)}</i>`)\n  })\n  _push(`<!--]-->`)\n  _push(`</div>`)\n}\n"
    );
    let map = with_map.map.expect("source_map should attach SSR map");
    assert_eq!(
        render(&with_map.code, source, &map),
        [
            r#"0:0 "function" -> "<div cla""#,
            r#"1:10 "div${_ss" -> "div clas""#,
            r#"1:45 "class: \"" -> "class=\"a""#,
            r#"1:53 "a\" }, _a" -> "a\"><p :t""#,
            r#"1:70 "p${_ssrR" -> "p :title""#,
            r#"1:89 "title\", " -> "title=\"t""#,
            r#"1:97 "_ctx.t)}" -> "t\">{{ ms" [t]"#,
            r#"1:124 "_ctx.msg" -> "msg }}</" [msg]"#,
            r#"2:2 "if (_ctx" -> "<b v-if=""#,
            r#"2:6 "_ctx.ok)" -> "ok\">Y</b" [ok]"#,
            r#"3:12 "b>Y</b>`" -> "b v-if=\"""#,
            r#"3:14 "Y</b>`)⏎" -> "Y</b><i ""#,
            r#"8:2 "_ssrRend" -> "<i v-for""#,
            r#"8:17 "_ctx.xs," -> "xs\">{{ x" [xs]"#,
            r#"9:12 "i>${_ssr" -> "i v-for=""#,
            r#"9:32 "x)}</i>`" -> "x }}</i>""#,
        ]
    );
}
