use super::{decode_mappings, line_column_at};
use crate::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc,
    parse_sfc,
};
use vize_carton::String;

fn offset_at(source: &str, line: usize, column: usize) -> usize {
    let line_start = source
        .split_inclusive('\n')
        .take(line)
        .map(str::len)
        .sum::<usize>();
    line_start + column
}

fn options() -> SfcCompileOptions {
    SfcCompileOptions {
        parse: SfcParseOptions {
            filename: String::from("Probe.vue"),
            ..Default::default()
        },
        script: ScriptCompileOptions {
            id: Some(String::from("Probe.vue")),
            ..Default::default()
        },
        template: TemplateCompileOptions {
            compiler_options: Some(vize_atelier_dom::DomCompilerOptions {
                source_map: true,
                ..Default::default()
            }),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn composed_maps_anchor_expression_in_final_code_and_original_sfc() {
    let cases = [
        r#"<template><div>{{ msg }}</div></template>
<script>export default { data: () => ({ msg: "hi" }) }</script>"#,
        r#"<template><div>{{ msg }}</div></template>
<script setup>const msg = "hi"</script>"#,
    ];

    for source in cases {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse");
        let result = compile_sfc(&descriptor, options()).expect("compile");
        let map = result.map.expect("composed map");
        let segments = decode_mappings(map["mappings"].as_str().unwrap()).unwrap();
        let source_offset = source.find("msg }}").expect("template expression");
        let source_position = line_column_at(source, source_offset);
        let mapping = segments
            .iter()
            .find(|segment| {
                segment.original.is_some_and(|original| {
                    (original.line as usize, original.column as usize) == source_position
                })
            })
            .expect("original expression anchor");
        let original = mapping.original.expect("original expression anchor");
        let generated_offset = offset_at(
            result.code.as_str(),
            mapping.generated_line,
            mapping.generated_column,
        );
        let generated_tail = &result.code[generated_offset..];

        assert_eq!(
            (original.line as usize, original.column as usize),
            source_position
        );
        assert!(
            generated_tail.starts_with("$data.msg")
                || generated_tail.starts_with("_ctx.msg")
                || generated_tail.starts_with("msg"),
            "mapped generated tail: {generated_tail:?}",
        );
        assert_eq!(map["sources"], serde_json::json!(["Probe.vue"]));
        assert_eq!(map["sourcesContent"], serde_json::json!([source]));
    }
}
