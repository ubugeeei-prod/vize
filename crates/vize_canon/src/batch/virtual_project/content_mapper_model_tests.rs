use std::path::Path;

use super::{
    CONTENT_MAPPER_SPAN_FEATURES_ALL, CONTENT_MAPPER_SPAN_FEATURES_COMPLETION,
    CONTENT_MAPPER_SPAN_FEATURES_WHOLE_SYMBOL, ContentMapperSpanKind,
    generate_vue_content_mapper_transform,
};
use crate::batch::ContentMapperTransform;

#[test]
fn maps_model_events_to_the_authored_model_name() {
    let source = r#"<script setup lang="ts">
defineModel<string>("title", { required: true });
</script>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("ModelChild.vue"), source).expect("model");
    let marker = "/* __vize_model_event */ \"update:title\"";
    let generated = result.text.find(marker).unwrap() + "/* __vize_model_event */ ".len();
    let original = source.find("\"title\"").unwrap();
    let prop_marker = "export type Props = {\n  \"title\"";
    let prop_generated = result.text.find(prop_marker).unwrap() + "export type Props = {\n  ".len();

    assert!(has_exact_mapping(&result, prop_generated, original));
    assert!(
        result.mappings.iter().any(|mapping| {
            mapping.0[0] == generated
                && mapping.0[1] == "\"update:title\"".len()
                && mapping.0[2] == original
                && mapping.0[3] == "\"title\"".len()
                && mapping.0[4] == ContentMapperSpanKind::Atom as usize
                && mapping.0[5] == CONTENT_MAPPER_SPAN_FEATURES_WHOLE_SYMBOL
        }),
        "text:\n{}\n\nmappings:\n{:#?}",
        result.text,
        result.mappings
    );

    let parent = r#"<script setup lang="ts">
import ModelChild from "./ModelChild.vue";
</script>
<template><ModelChild @update:title="() => undefined" /></template>
"#;
    let parent =
        generate_vue_content_mapper_transform(Path::new("App.vue"), parent).expect("parent");
    let completion = parent
        .text
        .find("__vize_model_events_completion_0['update:title']")
        .expect("completion projection")
        + "__vize_model_events_completion_0['".len();
    let navigation = parent
        .text
        .find("__vize_model_events_nav_0['update:title']")
        .expect("navigation projection")
        + "__vize_model_events_nav_0['".len();
    assert!(parent.mappings.iter().any(|mapping| {
        mapping.0[0] == completion && mapping.0[5] == CONTENT_MAPPER_SPAN_FEATURES_COMPLETION
    }));
    assert!(parent.mappings.iter().any(|mapping| {
        mapping.0[0] == navigation && mapping.0[5] == CONTENT_MAPPER_SPAN_FEATURES_WHOLE_SYMBOL
    }));
}

#[test]
fn maps_setup_scoped_model_props_to_the_authored_model_name() {
    let source = r#"<script setup lang="ts">
const schema = { title: "example" } as const;
defineProps<{ value: typeof schema }>();
defineModel<string>("title", { required: true });
</script>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("ScopedModel.vue"), source).expect("model");
    let marker = "type __VizeSetupProps = { value: typeof schema } & {\n  \"title\"";
    let generated = result.text.find(marker).expect("setup-scoped model prop") + marker.len()
        - "\"title\"".len();
    let original = source.find("\"title\"").unwrap();

    assert!(
        has_exact_mapping(&result, generated, original),
        "text:\n{}\n\nmappings:\n{:#?}",
        result.text,
        result.mappings
    );
}

#[test]
fn maps_models_merged_with_an_authored_public_props_type() {
    let source = r#"<script setup lang="ts">
export interface Props { value: string }
defineProps<Props>();
defineModel<string>("title", { required: true });
</script>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("PublicProps.vue"), source).expect("model");
    let marker = "type __VizeResolvedProps = Props & {\n  \"title\"";
    let generated =
        result.text.find(marker).expect("resolved model prop") + marker.len() - "\"title\"".len();
    let original = source.find("\"title\"").unwrap();

    assert!(
        has_exact_mapping(&result, generated, original),
        "text:\n{}\n\nmappings:\n{:#?}",
        result.text,
        result.mappings
    );
    assert!(
        result
            .text
            .contains("$props: __VizeComponentProps<__VizeResolvedProps> &")
    );
}

fn has_exact_mapping(result: &ContentMapperTransform, generated: usize, original: usize) -> bool {
    result.mappings.iter().any(|mapping| {
        mapping.0[0] == generated
            && mapping.0[1] == "\"title\"".len()
            && mapping.0[2] == original
            && mapping.0[3] == "\"title\"".len()
            && mapping.0[4] == ContentMapperSpanKind::Verbatim as usize
            && mapping.0[5] == CONTENT_MAPPER_SPAN_FEATURES_ALL
    })
}
