use std::path::Path;

use crate::batch::generate_vue_content_mapper_transform;

#[test]
fn art_variant_markup_is_projected_like_a_template() {
    let normal = r#"<script setup lang="ts">
import Child from './Child.vue'
const title = "heading"
</script>
<template>
  <Child :label="1" />
  <span>{{ title }}</span>
</template>
"#;
    let art = r#"<script setup lang="ts">
import Child from './Child.vue'
const title = "heading"
</script>
<art>
  <variant name="Broken">
    <Child :label="1" />
    <span>{{ title }}</span>
  </variant>
</art>
"#;
    let normal_ts = virtual_ts("Normal.vue", normal);
    let art_ts = virtual_ts("Button.art.vue", art);
    let needle = label_prop_needle(&normal_ts);
    assert!(
        art_ts.contains(needle),
        "variant markup must be in the checked template projection.\nnormal needle: {needle}\nart:\n{art_ts}"
    );
    assert!(
        art_ts.contains("title"),
        "script-setup bindings stay in scope for variant markup:\n{art_ts}"
    );
}

fn virtual_ts(filename: &str, source: &str) -> String {
    generate_vue_content_mapper_transform(Path::new(filename), source)
        .expect("transform")
        .text
        .to_string()
}

fn label_prop_needle(virtual_ts: &str) -> &str {
    for needle in ["\"label\": 1", "\"label\": (1)", "label: 1"] {
        if virtual_ts.contains(needle) {
            return needle;
        }
    }
    panic!("normal template did not project the label prop:\n{virtual_ts}");
}
