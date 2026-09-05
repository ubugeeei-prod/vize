use vize_glyph::{FormatOptions, format_sfc};

#[test]
fn templated_script_opening_tag_is_source_owned() {
    let source = r#"<script setup<%= useTs ? ' lang="ts"' : '' %>>
import { useData } from 'vitepress'

const { site, frontmatter } = useData()
</script>

<template>
  <h1>{{ site.title }}</h1>
</template>
"#;
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).expect("templated SFC must format");
    let second = format_sfc(&first.code, &options).expect("formatted SFC must stay parseable");
    let expected = r#"<script setup<%= useTs ? ' lang="ts"' : '' %>>
import { useData } from "vitepress";

const { site, frontmatter } = useData();
</script>

<template>
  <h1>{{ site.title }}</h1>
</template>
"#;

    assert_eq!(first.code, expected);
    assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
}
