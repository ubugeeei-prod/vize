use super::{ContentMapperTransformOptions, generate_vue_content_mapper_transform_with_options};
use std::path::Path;

#[test]
fn unused_diagnostic_setting_does_not_anchor_unread_destructured_props() {
    let source = r#"<script setup lang="ts">
const { label, color = "gray", ...rest } = defineProps<{ label: string; color?: string; title: string }>()
const unusedLocal = 1
</script>
<template><span>{{ label }} {{ rest.title }}</span></template>
"#;

    let result = generate_vue_content_mapper_transform_with_options(
        Path::new("MyBadge.vue"),
        source,
        ContentMapperTransformOptions::default().with_preserve_unused_diagnostics(true),
    )
    .expect("transform");

    assert_eq!(result.text.matches("void label;").count(), 1);
    assert_eq!(result.text.matches("void rest;").count(), 1);
    assert_eq!(result.text.matches("void color;").count(), 0);
    assert_eq!(result.text.matches("void unusedLocal;").count(), 0);
}

#[test]
fn used_props_rest_binding_is_anchored_before_the_template_shadow() {
    let source = r#"<script lang="ts">
type Foo = {
  a: string
  b: number
}

declare function func<TFoo extends Foo>(foo: TFoo, fooKey: keyof TFoo): any
</script>

<script setup lang="ts" generic="TFoo extends Foo, TFooKey extends keyof TFoo">
const { foo, fooKey, ...props } = defineProps<{
  foo: TFoo
  fooKey: TFooKey
  bar: TFoo
  barKey: TFooKey
}>()
</script>

<template>
  {{ func(foo, fooKey) }}
  {{ func(props.bar, props.barKey) }}
</template>
"#;

    let result = generate_vue_content_mapper_transform_with_options(
        Path::new("main.vue"),
        source,
        ContentMapperTransformOptions::default().with_preserve_unused_diagnostics(true),
    )
    .expect("transform");

    let anchor = result
        .text
        .find("// Anchor before the template scope shadows `props`\n  void props;")
        .expect("the authored rest binding must be read before template scope");
    let template = result
        .text
        .find("function __template()")
        .expect("template scope");
    assert!(
        anchor < template,
        "the in-template `props` binding is synthetic"
    );
}
