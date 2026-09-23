//! Exact span-carrying Vapor source maps (Davinci P3-9).
//!
//! Every test pins the full generated code and every decoded segment, rendered
//! as `generated text -> authored text [name]`, for both the legacy lowering
//! lane and the native S3 lane. The decoder is local on purpose: it must not
//! share code with the encoder it checks.

#![expect(clippy::string_slice, reason = "tests assert by panicking")]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "test fixtures and insta snapshots use std strings and format"
)]

use super::{
    VaporCompilerExperimentalOptions, VaporCompilerOptions, compile_vapor,
    compile_vapor_with_experimental_options,
};
use vize_carton::Allocator;

const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Which Vapor lane a test compile must take.
#[derive(Clone, Copy)]
enum Lane {
    /// Whatever the compile selects (the native S3 lane when it admits the
    /// template).
    Selected,
    /// The legacy lowering lane: binding metadata keeps the S3 bridge out.
    Legacy,
}

fn compile_on(source: &str, source_map: bool, lane: Lane) -> super::VaporCompileResult {
    let allocator = Allocator::new();
    let result = compile_vapor_with_experimental_options(
        &allocator,
        source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            binding_metadata: matches!(lane, Lane::Legacy).then(Default::default),
            ..VaporCompilerOptions::default()
        },
        VaporCompilerExperimentalOptions {
            source_map,
            source_map_filename: Some("Foo.vue".into()),
            ..VaporCompilerExperimentalOptions::default()
        },
    );
    assert_eq!(
        result.error_messages.len(),
        0,
        "{:?}",
        result.error_messages
    );
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

/// Compile with and without a map, require identical output, and render the
/// map's segments.
fn mapped(source: &str) -> (std::string::String, std::vec::Vec<std::string::String>) {
    mapped_on(source, Lane::Selected)
}

fn mapped_on(
    source: &str,
    lane: Lane,
) -> (std::string::String, std::vec::Vec<std::string::String>) {
    let without_map = compile_on(source, false, lane);
    let with_map = compile_on(source, true, lane);
    assert_eq!(
        (&with_map.code, &with_map.templates),
        (&without_map.code, &without_map.templates)
    );
    let map = with_map.map.expect("source_map should attach Vapor map");
    let segments = render(&with_map.code, source, &map);
    (with_map.code.as_str().into(), segments)
}

#[test]
fn source_map_disabled_by_default_yields_none() {
    let allocator = Allocator::new();
    let result = compile_vapor(
        &allocator,
        "<div>{{ msg }}</div>",
        VaporCompilerOptions::default(),
    );

    assert!(result.error_messages.is_empty(), "Expected no errors");
    assert!(result.map.is_none());
}

#[test]
fn both_lanes_map_templates_branches_and_loops_byte_exactly() {
    let source =
        "<ul>\n<li v-if=\"ok\">Y</li><li v-else>N</li><i v-for=\"x in xs\">{{ x }}</i>\n</ul>";
    // The native S3 lane admits this template; binding metadata forces the
    // legacy lane. Both must emit the same code with the same map.
    let (code, segments) = mapped(source);
    assert_eq!(
        mapped_on(source, Lane::Legacy),
        (code.clone(), segments.clone())
    );
    assert_eq!(
        code,
        "import { child as _child, next as _next, txt as _txt, toDisplayString as _toDisplayString, setText as _setText, setInsertionState as _setInsertionState, renderEffect as _renderEffect, createIf as _createIf, createFor as _createFor, template as _template } from 'vue';\nconst t0 = _template(\"<li>Y</li>\", true)\nconst t1 = _template(\"<li>N</li>\", true)\nconst t2 = _template(\"<i> </i>\")\nconst t3 = _template(\"<ul><!----><!----></ul>\", true)\n\nexport function render(_ctx) {\n  const n0 = t3()\n  const n1 = _child(n0)\n  const n7 = _next(_child(n0), 1)\n  _setInsertionState(n0, n1, true)\n  const n2 = _createIf(() => (_ctx.ok), () => {\n    const n4 = t0()\n    return n4\n  }, () => {\n    const n6 = t1()\n    return n6\n  })\n  _setInsertionState(n0, n7, true)\n  const n8 = _createFor(() => (_ctx.xs), (_for_item0) => {\n    const n10 = t2()\n    const x10 = _txt(n10)\n    _renderEffect(() => _setText(x10, _toDisplayString(_for_item0.value)))\n    return n10\n  })\n  return n0\n}\n"
    );
    assert_eq!(
        segments,
        [
            r#"1:23 "li>Y</li" -> "li v-if=""#,
            r#"1:26 "Y</li>\"," -> "Y</li><l""#,
            r#"2:23 "li>N</li" -> "li v-els""#,
            r#"2:26 "N</li>\"," -> "N</li><i""#,
            r#"3:23 "i> </i>\"" -> "i v-for=""#,
            r#"4:23 "ul><!---" -> "ul>⏎<li ""#,
            r#"6:0 "export f" -> "<ul>⏎<li""#,
            r#"11:13 "_createI" -> "<li v-if""#,
            r#"11:30 "_ctx.ok)" -> "ok\">Y</l" [ok]"#,
            r#"14:5 "() => {⏎" -> "<li v-el""#,
            r#"19:13 "_createF" -> "<i v-for""#,
            r#"19:31 "_ctx.xs)" -> "xs\">{{ x" [xs]"#,
            r#"22:55 "_for_ite" -> "x }}</i>""#,
        ]
    );
}

#[test]
fn native_lane_maps_templates_bindings_and_text_byte_exactly() {
    let source = "<main class=\"shell\"><span :title=\"hint\">{{ status }}</span></main>";
    let (code, segments) = mapped(source);
    assert_eq!(
        code,
        "import { child as _child, txt as _txt, toDisplayString as _toDisplayString, setText as _setText, setProp as _setProp, renderEffect as _renderEffect, template as _template } from 'vue';\nconst t0 = _template(\"<main class=\\\"shell\\\"><span> </span></main>\", true)\n\nexport function render(_ctx) {\n  const n0 = t0()\n  const n1 = _child(n0)\n  const x1 = _txt(n1)\n  _renderEffect(() => {\n    _setProp(n1, \"title\", _ctx.hint)\n    _setText(x1, _toDisplayString(_ctx.status))\n  })\n  return n0\n}\n"
    );
    assert_eq!(
        segments,
        [
            r#"1:23 "main cla" -> "main cla""#,
            r#"1:28 "class=\\\"" -> "class=\"s""#,
            r#"1:36 "shell\\\">" -> "shell\"><""#,
            r#"1:45 "span> </" -> "span :ti""#,
            r#"3:0 "export f" -> "<main cl""#,
            r#"8:18 "title\", " -> "title=\"h""#,
            r#"8:26 "_ctx.hin" -> "hint\">{{" [hint]"#,
            r#"9:34 "_ctx.sta" -> "status }" [status]"#,
        ]
    );
}

/// Template carriers, compound text and `v-model` admit on the native lane and
/// must keep the legacy lane's map. Component and slot props still carry no
/// native spans (P3-9).
#[test]
fn both_lanes_map_template_carriers_the_same() {
    for source in [
        r#"<div><template v-if="ok"><b>{{ a }}</b><i>x</i></template><span v-else>no</span></div>"#,
        r#"<ul><template v-for="item in items" :key="item.id"><li>{{ item.a }}</li><li>{{ item.b }}</li></template></ul>"#,
        r#"<template v-if="ok"><b>{{ a }}</b><i>x</i></template><template v-else>none</template>"#,
        r#"<main><template v-if="on">on {{ n }}</template><template v-else>off</template></main>"#,
        r#"<main><template v-for="x in xs">item {{ x }}</template></main>"#,
        r#"<ul><li v-if="a > b">Y</li><li v-else-if="c < d">N</li></ul>"#,
        r#"<input v-model="name">"#,
    ] {
        assert_eq!(
            mapped_on(source, Lane::Selected),
            mapped_on(source, Lane::Legacy),
            "{source}"
        );
    }
}
