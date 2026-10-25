//! Capture complete production checker and content-mapper projection packets.
//!
//! The constructor parity job runs this identical observer against the exact
//! PR base and head. Only nondeterministic analysis timing is excluded.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "qualification asserts by panicking"
)]

use std::path::Path;

use serde_json::{Value, json};
use vize_canon::sfc_typecheck::{
    SfcTypeCheckOptions, SfcTypeCheckResult, type_check_sfc, type_check_sfc_with_legacy_vue2,
    type_check_sfc_with_options_api,
};
use vize_canon::{
    ContentMapperTransformOptions, generate_vue_content_mapper_transform_with_options,
};

const GENERIC: &str = r#"<!-- 日本 🧭 'quoted' -->
<script setup lang="ts" generic="T extends { id: number; label: string } = { id: number; label: string }">
const props = defineProps<{ value: T }>()
const emit = defineEmits<{ change: [value: T] }>()
defineSlots<{ default: (props: { item: T }) => unknown }>()
defineExpose<{ selected: () => T }>()
const label = '日本 "quoted"'
function select() { emit('change', props.value) }
</script>
<template><button ref="target" @click="select">{{ label }}<slot :item="props.value" /></button></template>
"#;

const PLAIN: &str = r#"<!-- 日本 🧭 -->
<script setup lang="ts">
const props = defineProps<{ value: string }>()
defineEmits<{ change: [value: string] }>()
defineSlots<{ default: (props: { item: string }) => unknown }>()
defineExpose<{ selected: () => string }>()
</script>
<template><button ref="target"><slot :item="props.value" /></button></template>
"#;

const JAVASCRIPT: &str = "<script setup>const label = '日本 \\\"quoted\\\"'</script>\n<template><button>{{ label }}</button></template>\n";

type Checker = fn(&str, &SfcTypeCheckOptions) -> SfcTypeCheckResult;

fn checker_packet(source: &str, check: Checker, expects_projection: bool) -> Value {
    let options = SfcTypeCheckOptions::new("src/Constructor.vue").with_virtual_ts();
    let result = check(source, &options);
    assert_eq!(
        result.virtual_ts.is_some(),
        expects_projection,
        "source fixture must reach its intended production projection path"
    );
    let mut packet = serde_json::to_value(&result).unwrap();
    packet.as_object_mut().unwrap().remove("analysis_time_ms");
    packet["virtual_ts_mappings"] = Value::Array(result.virtual_ts_mappings.iter().map(|mapping| json!({
        "generated": mapping.gen_range,
        "authored": mapping.src_range,
        "subSpans": mapping.sub_spans.iter().map(|span| json!({"generated": span.gen_range, "authored": span.src_range})).collect::<Vec<_>>()
    })).collect());
    packet["virtual_ts_semantic_links"] = Value::Array(
        result
            .virtual_ts_semantic_links
            .iter()
            .map(|link| {
                json!({
                    "source": link.source_range,
                    "target": link.target_range,
                    "kind": vize_l0::cstr!("{:?}", link.kind),
                })
            })
            .collect(),
    );
    packet
}

#[test]
fn complete_constructor_projection_packets() {
    let invalid = GENERIC.replace("const label =", "const =");
    let repaired = GENERIC.replace("const label =", "const label = /* repaired */");
    let tsx = GENERIC.replace("lang=\"ts\"", "lang=\"tsx\"").replace(
        "const label =",
        "const render = () => <div>{props.value.label}</div>\nconst label =",
    );
    let cases = [
        ("generic", GENERIC),
        ("invalid", invalid.as_str()),
        ("repaired", repaired.as_str()),
        ("plain", PLAIN),
        ("javascript", JAVASCRIPT),
        ("tsx", tsx.as_str()),
    ];
    let mut rows = Vec::new();
    for newline in ["\n", "\r\n"] {
        for (name, input) in cases {
            let source = input.replace('\n', newline);
            for (target, check) in [
                ("vue3", type_check_sfc as Checker),
                ("options-api", type_check_sfc_with_options_api as Checker),
                ("legacy-vue2", type_check_sfc_with_legacy_vue2 as Checker),
            ] {
                rows.push(json!({"case": name, "newline": newline, "source": source, "target": target, "packet": checker_packet(&source, check, name != "invalid")}));
            }
            for bits in 0..16u8 {
                let options = ContentMapperTransformOptions::default()
                    .with_options_api(bits & 1 != 0)
                    .with_preserve_unused_diagnostics(bits & 2 != 0)
                    .with_check_javascript(bits & 4 != 0)
                    .with_external_component_attrs_module(bits & 8 != 0);
                let packet = generate_vue_content_mapper_transform_with_options(
                    Path::new("src/Constructor.vue"),
                    &source,
                    options,
                )
                .unwrap();
                assert_eq!(
                    packet.text.is_empty(),
                    false,
                    "mapper fixture must retain a complete production module, including syntax refusals"
                );
                rows.push(json!({"case": name, "newline": newline, "source": source, "target": "content-mapper", "options": bits, "packet": packet}));
            }
        }
    }
    assert_eq!(
        rows.len(),
        228,
        "every original source, newline and producer option must participate"
    );
    if let Some(path) = std::env::var_os("VIZE_CONSTRUCTOR_PROJECTION_CAPTURE") {
        std::fs::write(path, serde_json::to_vec(&rows).unwrap()).unwrap();
    }
    println!("constructor_projection_packets={}", rows.len());
}
