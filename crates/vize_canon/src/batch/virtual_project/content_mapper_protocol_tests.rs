use std::path::Path;

use serde_json::Value;

use super::super::protocol::protocol_semantic_links;
use super::{
    CONTENT_MAPPER_SPAN_FEATURE_BITS, CONTENT_MAPPER_SPAN_FEATURES_ALL,
    CONTENT_MAPPER_SPAN_FEATURES_ATOM, CONTENT_MAPPER_SPAN_FEATURES_COMPLETION,
    CONTENT_MAPPER_SPAN_FEATURES_NAVIGATION_TARGET, CONTENT_MAPPER_SPAN_FEATURES_WHOLE_SYMBOL,
    CONTENT_MAPPER_VIRTUAL_EXTENSION, ContentMapperSpanKind, content_mapper_span_features,
    generate_vue_content_mapper_transform,
};
use crate::batch::ContentMapperTransform;

#[test]
fn protocol_v1_omits_internal_component_prop_navigation_links() {
    use crate::virtual_ts::{VizeSemanticLink, VizeSemanticLinkKind};

    let links = [
        VizeSemanticLink {
            source_range: 1..6,
            target_range: 10..15,
            kind: VizeSemanticLinkKind::VueSetupTemplateRefUnwrap,
        },
        VizeSemanticLink {
            source_range: 20..25,
            target_range: 30..35,
            kind: VizeSemanticLinkKind::VueComponentPropNavigation,
        },
    ];

    let protocol = protocol_semantic_links(&links);

    assert_eq!(protocol.len(), 1);
    assert_eq!(protocol[0].kind, "vueSetupTemplateRefUnwrap");
}

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
fn protocol_v1_feature_constants_match_the_pinned_upstream_shape() {
    // Upstream `spanmap.FeatureAll` is `(FeatureCodeLens << 1) - 1` over 20 bits
    // and `SpanMap.Validate` rejects any segment carrying a bit outside it, so an
    // extra bit here is a hard transform failure, not a forward-compatible hint.
    assert_eq!(CONTENT_MAPPER_SPAN_FEATURES_ALL, 1_048_575);
    assert_ne!(CONTENT_MAPPER_SPAN_FEATURES_ALL, 3);
    assert_eq!(CONTENT_MAPPER_SPAN_FEATURE_BITS.len(), 20);

    let recomposed = CONTENT_MAPPER_SPAN_FEATURE_BITS
        .iter()
        .fold(0, |mask, (_, bit)| mask | bit);
    assert_eq!(recomposed, CONTENT_MAPPER_SPAN_FEATURES_ALL);
}

#[test]
fn transform_declares_its_virtual_extension_on_every_response() {
    // Upstream resolves the virtual syntax per transform response; an absent or
    // unsupported extension fails the whole file with TS100025.
    assert_eq!(CONTENT_MAPPER_VIRTUAL_EXTENSION, ".tsx");

    let valid = generate_vue_content_mapper_transform(
        Path::new("App.vue"),
        "<script setup lang=\"ts\">\nconst message = \"hello\"\n</script>\n<template>{{ message }}</template>\n",
    )
    .expect("transform");
    assert_eq!(valid.extension, CONTENT_MAPPER_VIRTUAL_EXTENSION);
    let valid_json = serde_json::to_value(&valid).expect("valid transform json");
    let valid_object = assert_serialized_transform_uses_protocol_v1_header(&valid_json);
    let mappings = valid_object["mappings"].as_array().expect("mappings array");
    assert!(!mappings.is_empty());
    assert!(mappings.iter().all(|mapping| {
        mapping
            .as_array()
            .is_some_and(|tuple| tuple.len() == 6 && tuple.iter().all(Value::is_number))
    }));

    // The unparseable-SFC fallback is still a successful transform response, so it
    // must declare an extension too.
    let fallback = generate_vue_content_mapper_transform(
        Path::new("Broken.vue"),
        "<template><div></template>\n<script setup lang=\"ts\">\n",
    )
    .expect("fallback transform");
    assert!(!fallback.diagnostics.is_empty());
    assert_eq!(fallback.extension, CONTENT_MAPPER_VIRTUAL_EXTENSION);
    let fallback_json = serde_json::to_value(&fallback).expect("fallback transform json");
    let fallback_object = assert_serialized_transform_uses_protocol_v1_header(&fallback_json);
    assert!(
        fallback_object["text"]
            .as_str()
            .is_some_and(|text| text.contains("__vize_component"))
    );
    assert!(
        fallback_object["mappings"]
            .as_array()
            .is_some_and(Vec::is_empty)
    );
    let diagnostic = fallback_object["diagnostics"]
        .as_array()
        .and_then(|diagnostics| diagnostics.first())
        .and_then(Value::as_object)
        .expect("fallback diagnostic object");
    for key in ["messageText", "start", "length", "code"] {
        assert!(diagnostic.contains_key(key), "{diagnostic:#?}");
    }
}

fn assert_serialized_transform_uses_protocol_v1_header(
    value: &Value,
) -> &serde_json::Map<String, Value> {
    let object = value.as_object().expect("transform object");
    assert_eq!(
        object["extension"].as_str(),
        Some(CONTENT_MAPPER_VIRTUAL_EXTENSION)
    );
    assert!(
        !object.contains_key("scriptKind"),
        "content-mapper protocol v1 uses extension, not the retired scriptKind field: {value:#}"
    );
    object
}

#[test]
fn feature_masks_have_positive_and_negative_bit_oracles() {
    assert_all_bits_enabled("verbatim all", CONTENT_MAPPER_SPAN_FEATURES_ALL);
    assert_exact_mask(
        "generic atom",
        CONTENT_MAPPER_SPAN_FEATURES_ATOM,
        &["Hover", "SignatureHelp"],
    );
    assert_exact_mask(
        "completion projection",
        CONTENT_MAPPER_SPAN_FEATURES_COMPLETION,
        &["Completion"],
    );
    assert_exact_mask(
        "whole-symbol alias",
        CONTENT_MAPPER_SPAN_FEATURES_WHOLE_SYMBOL,
        &[
            "Hover",
            "Definition",
            "TypeDefinition",
            "Implementation",
            "References",
            "DocumentHighlights",
        ],
    );
    assert_exact_mask(
        "navigation target",
        CONTENT_MAPPER_SPAN_FEATURES_NAVIGATION_TARGET,
        &[
            "Definition",
            "TypeDefinition",
            "Implementation",
            "References",
            "DocumentHighlights",
        ],
    );
    assert_exact_mask("diagnostic-only anchor", 0, &[]);
}

#[test]
fn emits_protocol_v1_spans_with_honest_features_and_without_forbidden_overlaps() {
    let source = r#"<script setup lang="ts">
const message = "hello"
</script>
<template>{{ message }}</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("App.vue"), source).expect("transform");

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
            == match mapping.0[4] {
                kind if kind == ContentMapperSpanKind::Verbatim as usize => {
                    CONTENT_MAPPER_SPAN_FEATURES_ALL
                }
                kind if kind == ContentMapperSpanKind::Atom as usize => {
                    CONTENT_MAPPER_SPAN_FEATURES_ATOM
                }
                kind if kind == ContentMapperSpanKind::Alias as usize => {
                    CONTENT_MAPPER_SPAN_FEATURES_WHOLE_SYMBOL
                }
                kind => panic!("unknown content-mapper span kind {kind}"),
            }
    }));
}

#[test]
fn classifies_vue_name_rewrites_as_aliases_without_edit_features() {
    let source = r#"<script setup lang="ts">
import Child from "./Child.vue";
const value = "example";
</script>
<template>
  <Child :foo-bar="value" v-model="value" @save-item="() => undefined" />
</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("Aliases.vue"), source).expect("transform");

    assert_alias_mapping(
        &result,
        "__vize_props_nav_0.fooBar",
        "__vize_props_nav_0.".len(),
        source.find(":foo-bar").unwrap() + 1,
        "foo-bar".len(),
    );
    assert_alias_mapping(
        &result,
        "__vize_props_nav_0.modelValue",
        "__vize_props_nav_0.".len(),
        source.find("v-model").unwrap(),
        "v-model".len(),
    );
    assert_alias_mapping(
        &result,
        "__vize_kebab_events_nav_0.saveItem",
        "__vize_kebab_events_nav_0.".len(),
        source.find("@save-item").unwrap() + 1,
        "save-item".len(),
    );
}

#[test]
fn keeps_arbitrary_synthetic_spans_as_atoms() {
    assert!(!super::super::alias::is_alias_projection(
        "__vize_handler_1_2",
        "handler"
    ));
    assert!(!super::super::alias::is_alias_projection(
        "props[\"class\"]",
        "class"
    ));
    assert!(!super::super::alias::is_alias_projection(
        "count + 1",
        "count"
    ));
}

#[test]
fn maps_generated_prop_check_identifiers_as_navigation_targets_without_hover_or_edits() {
    let source = r#"<script setup lang="ts">
import Child from "./Child.vue";
const value = 1;
</script>
<template><Child :foo-bar="value" /></template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("App.vue"), source).expect("transform");
    let original_start = source.find(":foo-bar").unwrap() + 1;

    let check_marker = "const __vize_prop_check_0_foo_bar";
    let check_generated_start = result
        .text
        .find(check_marker)
        .expect("prop check identifier")
        + "const ".len();
    assert!(
        result.mappings.iter().any(|mapping| {
            mapping.0[0] == check_generated_start
                && mapping.0[1] == "__vize_prop_check_0_foo_bar".len()
                && mapping.0[2] == original_start
                && mapping.0[3] == "foo-bar".len()
                && mapping.0[4] == ContentMapperSpanKind::Atom as usize
                && mapping.0[5] == CONTENT_MAPPER_SPAN_FEATURES_NAVIGATION_TARGET
        }),
        "expected navigation-target atom mapping for {check_marker}; text:\n{}\n\nmappings:\n{:#?}",
        result.text,
        result.mappings
    );
}

fn assert_all_bits_enabled(label: &str, mask: usize) {
    for (name, bit) in CONTENT_MAPPER_SPAN_FEATURE_BITS {
        assert_ne!(mask & bit, 0, "{label} should enable {name}");
    }
}

fn assert_exact_mask(label: &str, mask: usize, enabled_names: &[&str]) {
    for (name, bit) in CONTENT_MAPPER_SPAN_FEATURE_BITS {
        if enabled_names.contains(&name) {
            assert_ne!(mask & bit, 0, "{label} should enable {name}");
        } else {
            assert_eq!(mask & bit, 0, "{label} should not enable {name}");
        }
    }
}

fn assert_alias_mapping(
    result: &ContentMapperTransform,
    marker: &str,
    marker_offset: usize,
    original_start: usize,
    original_len: usize,
) {
    let generated_start = result.text.find(marker).expect(marker) + marker_offset;
    assert!(
        result.mappings.iter().any(|mapping| {
            mapping.0[0] == generated_start
                && mapping.0[2] == original_start
                && mapping.0[3] == original_len
                && mapping.0[4] == ContentMapperSpanKind::Alias as usize
                && mapping.0[5] == CONTENT_MAPPER_SPAN_FEATURES_WHOLE_SYMBOL
        }),
        "expected alias mapping for {marker}; text:\n{}\n\nmappings:\n{:#?}",
        result.text,
        result.mappings
    );
}
