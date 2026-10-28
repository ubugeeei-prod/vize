use std::path::Path;

use super::generate_vue_content_mapper_transform;

#[test]
fn event_navigation_suppresses_its_inference_only_callback_copy() {
    let source = r#"<script setup lang="ts">
import type { VNode } from "vue";
import Foo from "./Foo.vue";

const emit = defineEmits<{
  quit: [];
}>();
const slots = defineSlots<Partial<Record<"a" | "b", () => VNode>>>();
</script>

<template>
  <Foo :bar="(baz) => Boolean(slots[baz])" @quit="emit('quit')" />
</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("parent.vue"), source).expect("transform");
    let guarded = r#"// @ts-ignore Inference-only callback prop; mapped prop owner checks diagnostics.
    "bar": (baz) => Boolean(slots[baz]),"#;

    assert_eq!(
        result.text.matches(guarded).count(),
        3,
        "every inference-only callback copy must suppress its own line:\n{}",
        result.text
    );
    assert_eq!(
        result
            .text
            .matches(
                "// @ts-ignore Inference-only call; the authored prop checker owns diagnostics."
            )
            .count(),
        1,
        "the outer call still suppresses aggregate assignability errors:\n{}",
        result.text
    );
    assert_eq!(
        result
            .text
            .matches("= (baz) => Boolean(slots[baz]);")
            .count(),
        1,
        "the mapped authored check remains the sole unsuppressed diagnostic owner:\n{}",
        result.text
    );
}

#[test]
fn nested_function_type_arrow_keeps_the_outer_parameter_on_the_ignored_line() {
    let source = r#"<script setup lang="ts">
import Foo from "./Foo.vue";

const emit = defineEmits<{
  quit: [];
}>();
</script>

<template>
  <Foo
    :apply="(fn: (value: 'a' | 'b') => boolean,
      value) => fn(value)"
    @quit="emit('quit')"
  />
</template>
"#;
    let result =
        generate_vue_content_mapper_transform(Path::new("parent.vue"), source).expect("transform");
    let lines: Vec<_> = result.text.lines().collect();
    let guarded = lines
        .windows(2)
        .filter(|pair| {
            pair[0].contains(
                "@ts-ignore Inference-only callback prop; mapped prop owner checks diagnostics.",
            ) && pair[1].contains("\"apply\": (fn: (value: 'a' | 'b') => boolean,")
                && pair[1].contains("value) => fn(value),")
        })
        .count();

    assert_eq!(
        guarded, 3,
        "every inference-only copy must select the outer arrow and keep its parameter on the ignored line:\n{}",
        result.text
    );
    assert_eq!(
        result
            .text
            .lines()
            .filter(|line| line.contains("value) => fn(value);"))
            .count(),
        1,
        "the authored mapped owner remains the sole unsuppressed copy:\n{}",
        result.text
    );
}
