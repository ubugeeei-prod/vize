//! Block-tag attribute order and formatter presets (#6098).

use vize_glyph::{FormatOptions, FormatPreset, format_sfc};

#[test]
fn block_attributes_follow_sort_attributes() {
    let source = r#"<script lang="ts" setup>
const a = 1;
</script>

<template lang="html" functional>
  <div>{{ a }}</div>
</template>

<style module="css" lang="css" scoped>
.a {
  color: red;
}
</style>

<i18n locale="en" lang="json">
{}
</i18n>
"#;
    let authored = FormatOptions {
        sort_attributes: false,
        sort_blocks: false,
        ..FormatOptions::default()
    };
    let formatted = format_sfc(source, &authored).expect("format").code;
    assert_eq!(formatted.as_str(), source);

    let canonical = format_sfc(source, &FormatOptions::default())
        .expect("format")
        .code;
    assert!(
        canonical.contains("<script setup lang=\"ts\">"),
        "{canonical}"
    );
    assert!(
        canonical.contains("<template lang=\"html\" functional>"),
        "{canonical}"
    );
    assert!(
        canonical.contains("<style scoped lang=\"css\" module=\"css\">"),
        "{canonical}"
    );
    assert!(
        canonical.contains("<i18n lang=\"json\" locale=\"en\">"),
        "{canonical}"
    );
}

#[test]
fn prettier_preset_keeps_authored_order() {
    let source = r#"<template>
  <div v-bind:id="id" class="a">{{ id }}</div>
</template>

<script lang="ts" setup>
const id = "a";
</script>
"#;
    let mut options = FormatOptions::default();
    FormatPreset::Prettier.apply(&mut options);
    assert_eq!(options.print_width, 80);
    assert_eq!(
        format_sfc(source, &options).expect("format").code.as_str(),
        source
    );

    FormatPreset::Vize.apply(&mut options);
    let defaults = FormatOptions::default();
    assert_eq!(options.print_width, defaults.print_width);
    assert_eq!(options.sort_attributes, defaults.sort_attributes);
    assert_eq!(options.sort_blocks, defaults.sort_blocks);
    assert_eq!(
        options.normalize_directive_shorthands,
        defaults.normalize_directive_shorthands
    );
}
