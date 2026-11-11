use super::lint_sfc;

#[test]
fn commented_union_member_is_not_an_unused_prop() {
    let sfc = r#"<script setup lang="ts">
const { pet } = defineProps<{
  pet:
    | // cats
      Cat
    // dogs
    | Dog
    | Bird;
}>();
</script>
<template>
  <p>{{ pet.kind }}</p>
</template>
"#;
    let result = lint_sfc(sfc);
    assert!(
        result.diagnostics.is_empty(),
        "{:?}",
        result
            .diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect::<Vec<_>>()
    );
}
