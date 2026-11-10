//! Q16solver #11023: an unannotated template callback beside `@quit` must keep
//! its parameter context. The event resolver's inference copy is not the
//! diagnostic owner.

use std::path::Path;

use vize_canon::generate_vue_content_mapper_transform;

#[test]
fn template_callback_beside_listener_keeps_parameter_context() {
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
    let guarded = r#"// @ts-ignore
    "bar": (baz) => Boolean(slots[baz]),"#;

    assert_eq!(
        result.text.matches(guarded).count(),
        3,
        "every inference-only callback copy must suppress its own line:\n{}",
        result.text
    );
    let ignored_resolvers = result
        .text
        .lines()
        .zip(result.text.lines().skip(1))
        .filter(|(line, next)| {
            line.trim() == "// @ts-ignore"
                && next
                    .trim_start()
                    .starts_with("const __vize_events_resolved_")
        })
        .count();
    assert_eq!(
        ignored_resolvers, 1,
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
