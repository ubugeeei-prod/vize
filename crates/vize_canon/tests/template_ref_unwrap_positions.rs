//! Where a template reads a setup ref decides how `vue-tsc` unwraps it; these
//! pin the two positions the plain identifier unwrap missed, against the
//! upstream `unwrapBindingAccess` corpus project.

#[path = "support/script_block_project.rs"]
mod project;

const NO_DIAGNOSTICS: [&str; 0] = [];

/// A destructuring default is a template read: `fallback` is the unwrapped
/// `number`, not the setup scope's `Ref<number>`.
#[test]
fn destructured_loop_defaults_read_the_unwrapped_ref() {
    let source = r#"<script setup lang="ts">
import { ref } from 'vue';
const objs = ref([{ val: 'x' } as { val?: string }]);
const nested = ref([{ a: { b: 1 } as { b?: number } }]);
const fallback = ref(0);
const accept = (_: ACCEPTED) => {};
</script>
<template>
  <div v-for="{ val = fallback } in objs">{{ accept(val) }}</div>
  <div v-for="{ a: { b = fallback } } in nested">{{ accept(b) }}</div>
</template>"#;
    assert_eq!(
        project::check(&[(
            "src/App.vue",
            &source.replace("ACCEPTED", "string | number")
        )]),
        NO_DIAGNOSTICS
    );
    assert_eq!(
        project::check(&[("src/App.vue", &source.replace("ACCEPTED", "string"))]),
        [
            "src/App.vue(10,60): error TS2345: Argument of type 'number' is not assignable to parameter of type 'string'.",
            "src/App.vue(9,53): error TS2345: Argument of type 'string | number' is not assignable to parameter of type 'string'.\nType 'number' is not assignable to type 'string'.",
        ]
    );
}

/// A maybe-ref binding the template narrows anywhere is read through `.value`
/// everywhere, so the ref's own `value` absorbs the union's nullish member.
/// Without a narrowing read it keeps the distributive `number | undefined`.
#[test]
fn a_narrowed_maybe_ref_is_read_through_its_value_everywhere() {
    let source = r#"<script setup lang="ts">
import { inject, type Ref } from 'vue';
const injected = inject<Ref<number>>('count');
const accept = (_: number) => {};
</script>
<template>
  <p>{{ accept(injected) }}</p>
  <p v-if="GUARD">{{ accept(injected) }}</p>
</template>"#;
    assert_eq!(
        project::check(&[("src/App.vue", &source.replace("GUARD", "injected"))]),
        NO_DIAGNOSTICS
    );
    assert_eq!(
        project::check(&[("src/App.vue", &source.replace("GUARD", "true"))]),
        [
            "src/App.vue(7,16): error TS2345: Argument of type 'number | undefined' is not assignable to parameter of type 'number'.\nType 'undefined' is not assignable to type 'number'.",
            "src/App.vue(8,28): error TS2345: Argument of type 'number | undefined' is not assignable to parameter of type 'number'.\nType 'undefined' is not assignable to type 'number'.",
        ]
    );
}
