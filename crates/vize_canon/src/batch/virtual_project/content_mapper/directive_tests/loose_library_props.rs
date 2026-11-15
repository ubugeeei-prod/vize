use std::path::Path;

use super::super::{
    ContentMapperTransform, ContentMapperTransformOptions,
    generate_vue_content_mapper_transform_with_options,
};
use crate::virtual_ts::VirtualTsCheckOptions;

const SOURCE: &str = r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child :items="[1, 2]" :buffer="200" /></template>
"#;

fn transform(source: &str, check_unknown_props: bool) -> ContentMapperTransform {
    generate_vue_content_mapper_transform_with_options(
        Path::new("/project/App.vue"),
        source,
        ContentMapperTransformOptions {
            checks: VirtualTsCheckOptions {
                check_unknown_props,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("content mapper transform")
}

fn selected(transform: &ContentMapperTransform) -> Vec<[usize; 6]> {
    transform
        .diagnostic_directives
        .as_ref()
        .into_iter()
        .flat_map(|table| &table.directives)
        .filter(|directive| directive.diagnostic_code() == Some(2353))
        .map(|directive| directive.0)
        .collect()
}

#[test]
fn loose_mode_selects_only_ts2353_at_exact_component_literal_keys() {
    let result = transform(SOURCE, false);
    let expected = ["items", "buffer"]
        .map(|attribute| {
            let original = SOURCE.find(&format!(":{attribute}=")).unwrap() + 1;
            let generated_text = format!("\"{attribute}\": ");
            let generated = result
                .text
                .find(&generated_text)
                .unwrap_or_else(|| panic!("missing canonical {attribute} entry:\n{}", result.text));
            [
                original,
                attribute.len(),
                generated,
                generated + attribute.len() + 2,
                0,
                2353,
            ]
        })
        .to_vec();
    assert_eq!(selected(&result), expected, "{}", result.text);
}

#[test]
fn strict_config_and_sfc_comments_control_the_selector() {
    assert!(selected(&transform(SOURCE, true)).is_empty());

    let strict_comment = format!("<!-- @checkUnknownProps true -->\n{SOURCE}");
    assert!(selected(&transform(&strict_comment, false)).is_empty());

    let loose_comment = format!("<!-- @checkUnknownProps false -->\n{SOURCE}");
    assert!(!selected(&transform(&loose_comment, true)).is_empty());
}

#[test]
fn authored_directives_own_the_range_without_an_overlapping_selector() {
    for directive in ["@vue-ignore", "@vue-expect-error"] {
        let source = SOURCE.replace(
            "<template><Child",
            &format!("<template><!-- {directive} -->\n<Child"),
        );
        let result = transform(&source, false);
        assert!(
            selected(&result).is_empty(),
            "{directive}: {:#?}",
            result.diagnostic_directives
        );
    }
}
