//! Tests for the SFC module source map (#3399).

use super::test_support::{Segment, decode_mappings, descriptor_of};
use super::*;

/// A `.vue` whose script lines are all distinct, so every one of them is an
/// unambiguous anchor.
const COUNTER_VUE: &str = "<template>\n  <button @click=\"bump\">{{ count }}</button>\n</template>\n\n<script setup>\nimport { ref } from 'vue'\n\nconst count = ref(0)\nfunction bump() {\n  count.value += 1\n}\n</script>\n";

#[test]
fn map_document_and_every_segment_land_on_the_authored_lines() {
    let descriptor = descriptor_of(COUNTER_VUE);
    // Stand-in for the emitted module: the shape the SFC emitter produces —
    // hoisted template import, relocated user import, the setup body copied
    // verbatim, then synthesized render code that the source never contained.
    let generated = "import { toDisplayString as _toDisplayString } from \"vue\"\nimport { ref } from 'vue'\n\nexport default {\n  setup(__props) {\n\nconst count = ref(0)\nfunction bump() {\n  count.value += 1\n}\n\nreturn (_ctx, _cache) => {}\n}\n\n}\n";

    let json = build_sfc_source_map(generated, &descriptor, "/app/src/Counter.vue")
        .expect("a script-bearing SFC produces a map");
    let map: serde_json::Value = serde_json::from_str(&json).expect("map is valid JSON");

    assert_eq!(map["version"], serde_json::json!(3));
    assert_eq!(map["file"], serde_json::json!("/app/src/Counter.vue"));
    assert_eq!(map["sources"], serde_json::json!(["/app/src/Counter.vue"]));
    assert_eq!(map["sourcesContent"], serde_json::json!([COUNTER_VUE]));
    assert_eq!(map["names"], serde_json::json!([]));

    // Authored lines are 0-indexed: 0 `<template>`, 4 `<script setup>`,
    // 5 `import { ref } from 'vue'`, 7 `const count = ref(0)`,
    // 8 `function bump() {`, 9 `  count.value += 1`.
    assert_eq!(
        decode_mappings(map["mappings"].as_str().expect("mappings is a string")),
        vec![
            Segment {
                generated_line: 1,
                generated_column: 0,
                source_index: 0,
                source_line: 5,
                source_column: 0,
            },
            Segment {
                generated_line: 6,
                generated_column: 0,
                source_index: 0,
                source_line: 7,
                source_column: 0,
            },
            Segment {
                generated_line: 7,
                generated_column: 0,
                source_index: 0,
                source_line: 8,
                source_column: 0,
            },
            Segment {
                generated_line: 8,
                generated_column: 2,
                source_index: 0,
                source_line: 9,
                source_column: 2,
            },
        ],
    );
}

#[test]
fn re_indented_lines_keep_the_authored_column() {
    let source = "<script setup>\n    const deep = 1\n</script>\n";
    let descriptor = descriptor_of(source);
    // The emitter re-indents the statement to column 0; the anchor must still
    // report the authored column 4, not the generated one.
    let json = build_sfc_source_map("const deep = 1\n", &descriptor, "/a.vue")
        .expect("a re-indented copy still anchors");
    let map: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(
        decode_mappings(map["mappings"].as_str().unwrap()),
        vec![Segment {
            generated_line: 0,
            generated_column: 0,
            source_index: 0,
            source_line: 1,
            source_column: 4,
        }],
    );
}

#[test]
fn duplicated_and_synthesized_lines_are_left_unmapped() {
    // `count.value += 1` occurs twice, so it has no single origin; `_sfc_main`
    // plumbing occurs nowhere in the source. Only the unique line anchors.
    let source = "<script setup>\nconst only = 1\nfunction a() {\n  count.value += 1\n}\nfunction b() {\n  count.value += 1\n}\n</script>\n";
    let descriptor = descriptor_of(source);
    let generated = "const only = 1\n  count.value += 1\nconst _sfc_main = {}\n";

    let json = build_sfc_source_map(generated, &descriptor, "/a.vue").unwrap();
    let map: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(
        decode_mappings(map["mappings"].as_str().unwrap()),
        vec![Segment {
            generated_line: 0,
            generated_column: 0,
            source_index: 0,
            source_line: 1,
            source_column: 0,
        }],
    );
}

#[test]
fn template_and_style_text_is_never_an_anchor() {
    // The generated line is byte-identical to a `<template>` line, but the
    // index only covers script blocks, so nothing anchors and no map is made.
    let source =
        "<template>\n  const fake = 1\n</template>\n<script setup>\nconst real = 2\n</script>\n";
    let descriptor = descriptor_of(source);

    assert_eq!(
        build_sfc_source_map("const fake = 1\n", &descriptor, "/a.vue"),
        None,
    );
}

#[test]
fn script_less_sfc_produces_no_map() {
    let descriptor = descriptor_of("<template>\n  <p>hi</p>\n</template>\n");

    assert_eq!(
        build_sfc_source_map("export default {}\n", &descriptor, "/a.vue"),
        None,
    );
}

#[test]
fn structural_punctuation_is_not_anchorable() {
    assert!(!is_anchorable("}"));
    assert!(!is_anchorable("})"));
    assert!(!is_anchorable("});"));
    assert!(!is_anchorable("[]"));
    assert!(is_anchorable("a++"));
    assert!(is_anchorable("const only = 1"));
}

#[test]
fn crlf_sources_resolve_to_the_same_positions() {
    let source = "<script setup>\r\nconst crlf = 1\r\n</script>\r\n";
    let descriptor = descriptor_of(source);

    let json = build_sfc_source_map("const crlf = 1\r\n", &descriptor, "/a.vue").unwrap();
    let map: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(
        decode_mappings(map["mappings"].as_str().unwrap()),
        vec![Segment {
            generated_line: 0,
            generated_column: 0,
            source_index: 0,
            source_line: 1,
            source_column: 0,
        }],
    );
}

#[cfg(feature = "compile")]
fn compile_with_source_map(source: &str, source_map: bool) -> crate::types::SfcCompileResult {
    use crate::compile_sfc_with_template_syntax_and_codegen_options;
    use crate::types::{ScriptCompileOptions, SfcCompileOptions, SfcParseOptions};
    use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode};

    let descriptor = descriptor_of(source);
    let options = SfcCompileOptions {
        parse: SfcParseOptions {
            filename: "/app/src/Counter.vue".into(),
            ..Default::default()
        },
        script: ScriptCompileOptions {
            id: Some("/app/src/Counter.vue".into()),
            inline_template: false,
            ..Default::default()
        },
        ..Default::default()
    };
    compile_sfc_with_template_syntax_and_codegen_options(
        &descriptor,
        options,
        TemplateSyntaxMode::Standard,
        CodegenOptions {
            source_map,
            ..Default::default()
        },
    )
    .expect("fixture compiles")
}

#[cfg(feature = "compile")]
#[test]
fn compile_sfc_attaches_a_map_only_when_the_flag_is_on() {
    assert_eq!(compile_with_source_map(COUNTER_VUE, false).map, None);

    let result = compile_with_source_map(COUNTER_VUE, true);
    let map = result.map.clone().expect("source_map: true attaches a map");

    assert_eq!(map["sources"], serde_json::json!(["/app/src/Counter.vue"]));
    assert_eq!(map["sourcesContent"], serde_json::json!([COUNTER_VUE]));

    // The emitted module really does carry the authored statement, and its
    // segment resolves back to authored line 7, column 0.
    let generated_line = result
        .code
        .lines()
        .position(|line| line == "const count = ref(0)")
        .expect("the emitter copies the authored statement verbatim");
    let segments = decode_mappings(map["mappings"].as_str().unwrap());
    assert_eq!(
        segments
            .iter()
            .find(|segment| segment.generated_line == generated_line),
        Some(&Segment {
            generated_line,
            generated_column: 0,
            source_index: 0,
            source_line: 7,
            source_column: 0,
        }),
    );
}
