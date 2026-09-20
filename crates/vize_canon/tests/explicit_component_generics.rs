//! `<!-- @vue-generic {…} -->` instantiates the component usage it precedes
//! with the authored type arguments (the upstream `v-generic` corpus project).

#[path = "support/script_block_project.rs"]
mod project;

const CHILD: &str = r#"<script setup lang="ts" generic="T extends string | number, K = any">
defineProps<{ value?: T }>();
defineEmits<{ foo: [bar: T, baz: K] }>();
</script>
"#;

#[test]
fn explicit_arguments_type_listeners_and_props_and_report_at_the_comment() {
    let parent = r#"<script setup lang="ts">
import Comp from './Comp.vue';
const acceptString = (_: string) => {};
const acceptBoolean = (_: boolean) => {};
</script>
<template>
  <!-- @vue-generic {string, boolean} -->
  <Comp @foo="(bar, baz) => (acceptString(bar), acceptBoolean(baz))" />
  <!-- @vue-generic {number} -->
  <Comp :value="'text'" />
  <!-- @vue-generic {boolean} -->
  <Comp />
  <!-- @vue-expect-error -->
  <!-- @vue-generic {boolean} -->
  <Comp />
</template>
"#;
    assert_eq!(
        project::check(&[("src/Comp.vue", CHILD), ("src/App.vue", parent)]),
        [
            "src/App.vue(10,10): error TS2322: Type 'string' is not assignable to type 'number'.",
            "src/App.vue(11,22): error TS2344: Type 'boolean' does not satisfy the constraint 'string | number'.",
        ]
    );
}

/// Without the comment the listener parameters fall back to the constraints,
/// which is what makes the explicit arguments observable above.
#[test]
fn inferred_arguments_fall_back_to_the_type_parameter_constraints() {
    let parent = r#"<script setup lang="ts">
import Comp from './Comp.vue';
const acceptString = (_: string) => {};
</script>
<template>
  <Comp @foo="(bar) => acceptString(bar)" />
</template>
"#;
    assert_eq!(
        project::check(&[("src/Comp.vue", CHILD), ("src/App.vue", parent)]),
        [
            "src/App.vue(6,37): error TS2345: Argument of type 'string | number' is not assignable to parameter of type 'string'.\nType 'number' is not assignable to type 'string'."
        ]
    );
}
