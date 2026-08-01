//! Tests for the anchor rules that survive an oxc re-print (#3399).

use super::test_support::{Segment, decode_mappings, descriptor_of};
use super::*;

/// A `lang="ts"` SFC: oxc re-prints the whole module before it leaves the napi
/// boundary, so no emitted line is byte-identical to the authored one. The
/// printer-normalised and declared-binding rules have to carry it.
const TS_VUE: &str = "<template><p>{{ msg }}</p></template>\n<script setup lang=\"ts\">\nconst msg: string = 'hello'\nfunction shout(): string {\n  return msg.toUpperCase()\n}\n</script>\n";

#[test]
fn oxc_reprinted_typescript_still_anchors_on_the_authored_lines() {
    let descriptor = descriptor_of(TS_VUE);
    // Verbatim oxc output shape: double quotes, semicolons, re-indentation,
    // type annotations gone.
    let generated = "import { toDisplayString as _toDisplayString } from \"vue\";\nconst msg = \"hello\";\nexport default {\n  __name: \"ts\",\n  setup(__props) {\n    function shout() {\n      return msg.toUpperCase();\n    }\n    return (_ctx, _cache) => {};\n  }\n};\n";

    let json = build_sfc_source_map(generated, &descriptor, "/a.vue")
        .expect("a re-printed TypeScript module still anchors");
    let map: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Authored lines: 2 `const msg: string = 'hello'`, 3 `function shout(): string {`,
    // 4 `  return msg.toUpperCase()`.
    assert_eq!(
        decode_mappings(map["mappings"].as_str().unwrap()),
        vec![
            Segment {
                generated_line: 1,
                generated_column: 0,
                source_index: 0,
                source_line: 2,
                source_column: 0,
            },
            Segment {
                generated_line: 5,
                generated_column: 4,
                source_index: 0,
                source_line: 3,
                source_column: 0,
            },
            Segment {
                generated_line: 6,
                generated_column: 6,
                source_index: 0,
                source_line: 4,
                source_column: 2,
            },
        ],
    );
}

#[test]
fn normalized_key_folds_only_what_a_printer_changes() {
    assert_eq!(
        normalized_key("return msg.toUpperCase();"),
        "return msg.toUpperCase()"
    );
    assert_eq!(normalized_key("const a  =   'x' ;"), "const a = \"x\"");
    assert_eq!(normalized_key("if (a) {"), "if (a) {");
}

#[test]
fn declaration_key_names_the_binding_a_line_introduces() {
    assert_eq!(
        declaration_key("const msg: string = 'x'"),
        Some(("const", "msg"))
    );
    assert_eq!(
        declaration_key("export const msg = 1"),
        Some(("const", "msg"))
    );
    assert_eq!(
        declaration_key("export default async function run() {"),
        Some(("function", "run"))
    );
    assert_eq!(
        declaration_key("function* gen() {"),
        Some(("function", "gen"))
    );
    assert_eq!(declaration_key("class Widget {"), Some(("class", "Widget")));
    assert_eq!(declaration_key("constant = 5"), None);
    assert_eq!(declaration_key("return msg"), None);
}

#[test]
fn a_generated_declaration_maps_to_the_authored_declaration_of_the_same_binding() {
    // `const props = defineProps(...)` is emitted as `const props = __props`;
    // both declare `props`, so the binding rule anchors them together.
    let source = "<script setup>\nconst props = defineProps({ id: String })\n</script>\n";
    let descriptor = descriptor_of(source);

    let json = build_sfc_source_map("const props = __props\n", &descriptor, "/a.vue").unwrap();
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
