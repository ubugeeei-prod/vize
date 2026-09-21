//! Exact span-carrying emission tests for the DOM codegen (Davinci P3-9).
//!
//! Each test pins the full generated code, the full `names` array and every
//! decoded segment, rendered as `generated token -> authored token` so a
//! reviewer can check each anchor without decoding VLQ by hand.

use super::tests::{DecodedSegment, compile_with_map, decode_mappings};

/// Render every segment as `generated line:column "generated text" -> authored
/// "authored text" [name]`, taking `width` bytes on each side.
fn render(code: &str, source: &str, map: &str, width: usize) -> Vec<std::string::String> {
    let parsed: serde_json::Value = serde_json::from_str(map).expect("map is JSON");
    let names = parsed["names"].as_array().expect("names array");
    let line_offset = |text: &str, line: u32, column: u32| {
        let start: usize = text
            .split_inclusive('\n')
            .take(line as usize)
            .map(str::len)
            .sum();
        start + column as usize
    };
    let window = |text: &str, offset: usize| {
        let end = (offset + width).min(text.len());
        text[offset..end].replace('\n', "⏎")
    };
    decode_mappings(parsed["mappings"].as_str().expect("mappings"))
        .into_iter()
        .map(
            |DecodedSegment {
                 generated_line,
                 generated_column,
                 source_line,
                 source_column,
                 name,
             }| {
                let generated = line_offset(code, generated_line, generated_column);
                let authored = line_offset(source, source_line, source_column);
                let name = name.map_or(std::string::String::new(), |index| {
                    std::format!(" [{}]", names[index as usize].as_str().expect("name"))
                });
                std::format!(
                    "{generated_line}:{generated_column} {:?} -> {:?}{name}",
                    window(code, generated),
                    window(source, authored),
                )
            },
        )
        .collect()
}

#[test]
fn rewritten_identifiers_tags_and_render_entry_map_byte_exactly() {
    let source = r#"<p :title="tip">{{ count + offset }}</p>"#;
    let result = compile_with_map(source, "Foo.vue");
    assert_eq!(
        result.code.as_str(),
        "function render(_ctx, _cache, $props, $setup, $data, $options) {\n  return (_openBlock(), _createElementBlock(\"p\", { title: _ctx.tip }, _toDisplayString(_ctx.count + _ctx.offset), 9 /* TEXT, PROPS */, [\"title\"]))\n}"
    );
    assert_eq!(
        render(&result.code, source, result.map.as_deref().expect("map"), 8),
        [
            r#"0:0 "function" -> "<p :titl""#,
            r#"1:9 "(_openBl" -> "<p :titl""#,
            r#"1:45 "p\", { ti" -> "p :title""#,
            r#"1:51 "title: _" -> "title=\"t" [title]"#,
            r#"1:58 "_ctx.tip" -> "tip\">{{ " [tip]"#,
            r#"1:87 "_ctx.cou" -> "count + " [count]"#,
            r#"1:100 "_ctx.off" -> "offset }" [offset]"#,
        ]
    );
}

#[test]
fn branches_loops_slots_and_their_text_map_byte_exactly() {
    let source = "<Card><template #head>Hi</template></Card>\n<ul><li v-if=\"ok\">Yes</li><li v-else>No</li><i v-for=\"x in xs\">{{ x }}</i></ul>";
    let result = compile_with_map(source, "Foo.vue");
    assert_eq!(
        result.code.as_str(),
        "function render(_ctx, _cache, $props, $setup, $data, $options) {\n  const _component_Card = _resolveComponent(\"Card\")\n  \n  return (_openBlock(), _createElementBlock(_Fragment, null, [\n    _createVNode(_component_Card, null, {\n      head: _withCtx(() => [\n        _createTextVNode(\"Hi\")\n      ]),\n      _: 1 /* STABLE */\n    }),\n    _createElementVNode(\"ul\", null, [\n      (_ctx.ok)\n        ? (_openBlock(), _createElementBlock(\"li\", { key: 0 }, \"Yes\"))\n        : (_openBlock(), _createElementBlock(\"li\", { key: 1 }, \"No\")),\n      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.xs, (x) => {\n        return (_openBlock(), _createElementBlock(\"i\", null, _toDisplayString(x), 1 /* TEXT */))\n      }), 256 /* UNKEYED_FRAGMENT */))\n    ])\n  ], 64 /* STABLE_FRAGMENT */))\n}"
    );
    assert_eq!(
        render(&result.code, source, result.map.as_deref().expect("map"), 8),
        [
            r#"0:0 "function" -> "<Card><t""#,
            r#"1:45 "Card\")⏎ " -> "Card><te""#,
            r#"5:6 "head: _w" -> "head>Hi<""#,
            r#"5:12 "_withCtx" -> "<templat""#,
            r#"6:26 "Hi\")⏎   " -> "Hi</temp""#,
            r#"10:25 "ul\", nul" -> "ul><li v""#,
            r#"11:7 "_ctx.ok)" -> "ok\">Yes<" [ok]"#,
            r#"12:10 "(_openBl" -> "<li v-if""#,
            r#"12:46 "li\", { k" -> "li v-if=""#,
            r#"12:64 "Yes\"))⏎ " -> "Yes</li>""#,
            r#"13:10 "(_openBl" -> "<li v-el""#,
            r#"13:46 "li\", { k" -> "li v-els""#,
            r#"13:64 "No\")),⏎ " -> "No</li><""#,
            r#"14:6 "(_openBl" -> "<i v-for""#,
            r#"14:74 "_ctx.xs," -> "xs\">{{ x" [xs]"#,
            r#"15:51 "i\", null" -> "i v-for=""#,
            r#"15:78 "x), 1 /*" -> "x }}</i>""#,
        ]
    );
}
