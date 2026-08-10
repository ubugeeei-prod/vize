use std::path::Path;

use super::ContentMapperSpanKind;
use super::span_features::{
    CONTENT_MAPPER_SPAN_FEATURES_ALL, CONTENT_MAPPER_SPAN_FEATURES_ATOM,
    CONTENT_MAPPER_SPAN_FEATURES_COMPLETION, CONTENT_MAPPER_SPAN_FEATURES_WHOLE_SYMBOL,
    content_mapper_span_features,
};
use crate::batch::{
    ContentMapperTransformOptions, generate_vue_content_mapper_transform,
    generate_vue_content_mapper_transform_with_options,
};

#[path = "content_mapper_component_export_tests.rs"]
mod component_exports;
#[path = "content_mapper_model_tests.rs"]
mod models;
#[path = "content_mapper_navigation_tests.rs"]
mod navigation;
#[path = "content_mapper_scoped_event_navigation_tests.rs"]
mod scoped_event_navigation;

#[test]
fn keeps_diagnostic_handler_anchors_out_of_editor_features() {
    let generated = "  const __vize_handler_1_2: unknown = handler;";
    let start = generated.find("__vize_handler_").unwrap();

    assert_eq!(
        content_mapper_span_features(generated, start, ContentMapperSpanKind::Atom),
        0
    );
    assert_eq!(
        content_mapper_span_features(generated, start, ContentMapperSpanKind::Verbatim),
        CONTENT_MAPPER_SPAN_FEATURES_ALL
    );
}

#[test]
fn emits_protocol_v1_spans_with_all_features_and_without_forbidden_overlaps() {
    let source = r#"<script setup lang="ts">
const message = "hello"
</script>
<template>{{ message }}</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("App.vue"), source).expect("transform");

    assert_eq!(result.script_kind, 3);
    assert!(result.text.contains("const message = \"hello\""));
    assert!(
        !result
            .text
            .contains("/// <reference types=\"vite/client\" />")
    );
    assert!(!result.mappings.is_empty());
    for pair in result.mappings.windows(2) {
        let left = pair[0].0;
        let right = pair[1].0;
        assert!(left[0] + left[1] <= right[0], "{left:?} overlaps {right:?}");
    }
    for (index, left) in result.mappings.iter().enumerate() {
        for right in &result.mappings[index + 1..] {
            let left = left.0;
            let right = right.0;
            let originals_overlap = left[2] < right[2] + right[3] && right[2] < left[2] + left[3];
            let originals_match = left[2] == right[2] && left[3] == right[3];
            assert!(
                !originals_overlap || originals_match,
                "{left:?} partially overlaps {right:?}"
            );
        }
    }
    assert!(result.mappings.iter().all(|mapping| {
        mapping.0[5]
            == if mapping.0[4] == ContentMapperSpanKind::Verbatim as usize {
                CONTENT_MAPPER_SPAN_FEATURES_ALL
            } else {
                CONTENT_MAPPER_SPAN_FEATURES_ATOM
            }
    }));
}

#[test]
fn keeps_mapper_offsets_in_utf8_bytes() {
    let source = r#"<script setup lang="ts">
const emoji = "😀"
</script>
<template>{{ emoji }}</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("Unicode.vue"), source).expect("transform");
    let original = source.rfind("emoji").expect("template identifier");
    assert!(
        result
            .mappings
            .iter()
            .any(|mapping| mapping.0[2] == original),
        "expected a UTF-8 byte mapping at {original}: {:?}",
        result.mappings
    );
}

#[test]
fn maps_synthetic_prop_bindings_to_the_authored_declaration() {
    let source = r#"<script setup lang="ts">
defineProps<{ count: number }>();
</script>
<template>{{ count.toFixed(0) }}</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("Props.vue"), source).expect("transform");
    let original = source.find("count: number").unwrap();
    let matching = result
        .mappings
        .iter()
        .filter(|mapping| mapping.0[2] == original && mapping.0[3] == "count".len())
        .collect::<Vec<_>>();
    let exported = result.text.find("export type Props").unwrap();
    let exported = exported + result.text[exported..].find("count").unwrap();

    assert!(
        matching.len() >= 3,
        "expected exported, authored, and synthetic projections: {matching:?}"
    );
    assert!(matching.iter().any(|mapping| mapping.0[0] == exported));
    assert!(matching.iter().all(|mapping| mapping.0[4] == 0));
}

#[test]
fn maps_synthetic_props_after_a_plain_script_to_the_setup_block() {
    let source = r#"<script lang="ts">
export const marker = true;
</script>
<script setup lang="ts">
defineProps<{ count: number }>();
</script>
<template>{{ count.toFixed(0) }}</template>
"#;
    let result = generate_vue_content_mapper_transform(Path::new("SplitProps.vue"), source)
        .expect("transform");
    let original = source.find("count: number").unwrap();

    assert!(
        result
            .mappings
            .iter()
            .filter(|mapping| {
                mapping.0[2] == original && mapping.0[3] == "count".len() && mapping.0[4] == 0
            })
            .count()
            >= 2
    );
}

#[test]
fn split_script_setup_spans_start_at_the_authored_block() {
    let source = r#"<script lang="ts">
export type SearchQuery = { value: string };
</script>

<script setup lang="ts">
const values: any = [];
values.map(it => it);
</script>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("Split.vue"), source).expect("transform");
    let generated = result.text.find("it => it").expect("generated parameter");
    let original = source.find("it => it").expect("authored parameter");
    let span = result
        .mappings
        .iter()
        .map(|mapping| mapping.0)
        .find(|span| generated >= span[0] && generated < span[0] + span[1])
        .expect("parameter mapping");

    assert_eq!(span[4], 0, "setup line should remain verbatim: {span:?}");
    assert_eq!(span[2] + generated - span[0], original);
}

#[test]
fn authored_parse_errors_are_mapper_diagnostics() {
    let source = "<template><div></template>";
    let result =
        generate_vue_content_mapper_transform(Path::new("Broken.vue"), source).expect("transform");

    assert!(!result.diagnostics.is_empty());
    assert!(result.mappings.is_empty());
    assert!(result.text.contains("__vize_component"));
    assert!(result.diagnostics[0].start <= source.len());
}

/// Opening line of the generated template scope.
const TEMPLATE_SCOPE: &str = "  ;(function __template() {\n";

/// The first four lines the template scope emits for a Vue 3 component that has
/// a setup binding to shadow, when the shared preamble is not hoisted.
const TEMPLATE_REF_UNWRAP_PRELUDE: &str = r#"    // Auto-unwrap Vue refs in template scope
    type __VizeIsUnion<T, __U = T> = T extends unknown ? ([__U] extends [T] ? false : true) : false;
    type __VizeWidenTemplateRef<T> = __VizeIsUnion<T> extends true ? T : T extends string ? keyof T extends keyof string ? string : T : T extends number ? keyof T extends keyof number ? number : T : T extends boolean ? keyof T extends keyof boolean ? boolean : T : T;
    type __U<T> = T extends import('vue').Ref ? __VizeWidenTemplateRef<T['value']> : T;
"#;

fn template_scope_of(text: &str) -> &str {
    text.split_once(TEMPLATE_SCOPE)
        .expect("generated module must open a template scope")
        .1
}

/// This path does not hoist the shared preamble, so every helper it emits is a
/// module-local declaration. The widening conditional types must therefore stay
/// with the `__U` that is their only reference, inside `__template()`: a
/// component with no setup binding in template scope emits no `__U` at all, and
/// a module-scope copy would then be unused — which TypeScript reports as a
/// TS6196 hint on the user's own `.vue` file (#3510).
#[test]
fn widening_helpers_are_declared_with_the_template_scope_that_uses_them() {
    let source = r#"<script setup lang="ts">
import { ref } from 'vue'
const message = ref('hello')
</script>
<template>{{ message }}</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("Unwrap.vue"), source).expect("transform");

    let prelude = template_scope_of(&result.text)
        .split_inclusive('\n')
        .take(4)
        .collect::<std::string::String>();
    assert_eq!(prelude, TEMPLATE_REF_UNWRAP_PRELUDE);
    for declaration in ["type __VizeIsUnion<", "type __VizeWidenTemplateRef<"] {
        assert_eq!(
            result.text.matches(declaration).count(),
            1,
            "{declaration} must be declared once, in template scope:\n{}",
            result.text
        );
    }
}

/// The other half of the same invariant: nothing to unwrap, nothing declared.
#[test]
fn a_component_without_setup_bindings_declares_no_widening_helpers() {
    let source = "<template><p>static</p></template>\n";
    let result =
        generate_vue_content_mapper_transform(Path::new("Static.vue"), source).expect("transform");

    for declaration in [
        "type __U<",
        "type __VizeIsUnion<",
        "type __VizeWidenTemplateRef<",
    ] {
        assert_eq!(
            result.text.matches(declaration).count(),
            0,
            "{declaration} would be an unused module-local declaration:\n{}",
            result.text
        );
    }
}

#[test]
fn jsx_scripts_report_tsx_script_kind() {
    let source = "<script lang=\"tsx\">export default () => <div /></script>";
    let result =
        generate_vue_content_mapper_transform(Path::new("Jsx.vue"), source).expect("transform");

    assert_eq!(result.script_kind, 4);
    assert!(
        result
            .text
            .starts_with("/// <reference types=\"vue/jsx\" />")
    );
}

#[test]
fn options_api_transform_setting_controls_instance_bindings() {
    let source = r#"<script lang="ts">
export default {
  data() { return { count: 1 } }
}
</script>
<template>{{ count }}</template>
"#;

    let enabled = generate_vue_content_mapper_transform_with_options(
        Path::new("Options.vue"),
        source,
        ContentMapperTransformOptions::default().with_options_api(true),
    )
    .expect("enabled transform");
    let disabled = generate_vue_content_mapper_transform_with_options(
        Path::new("Options.vue"),
        source,
        ContentMapperTransformOptions::default().with_options_api(false),
    )
    .expect("disabled transform");

    assert!(
        enabled
            .text
            .contains("const count: __VizeOptionsBinding<typeof __default__, \"count\">")
    );
    assert!(!disabled.text.contains("__VizeOptionsBinding"));
}

#[test]
fn unused_diagnostic_setting_only_anchors_template_references() {
    let source = r#"<script setup lang="ts">
const used = 1
const unused = 2
</script>
<template>{{ used }}</template>
"#;

    let result = generate_vue_content_mapper_transform_with_options(
        Path::new("Unused.vue"),
        source,
        ContentMapperTransformOptions::default().with_preserve_unused_diagnostics(true),
    )
    .expect("transform");

    assert!(result.text.contains("void used;"), "{}", result.text);
    assert!(!result.text.contains("void unused;"), "{}", result.text);
}

#[test]
fn default_transform_matches_vize_options_api_default() {
    let source = r#"<script lang="ts">
export default { data() { return { count: 1 } } }
</script>
<template>{{ count }}</template>
"#;

    let result = generate_vue_content_mapper_transform(Path::new("DefaultOptions.vue"), source)
        .expect("transform");

    assert!(result.text.contains("__VizeOptionsBinding"));
}
