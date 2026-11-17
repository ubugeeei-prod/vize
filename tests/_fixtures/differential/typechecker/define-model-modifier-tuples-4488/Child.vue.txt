<script setup lang="ts">
const [model, modifiers] = defineModel<string, "trim" | "capitalize">({ required: true });

if (modifiers.trim) {
  model.value = model.value.trim();
}
</script>

<template>
  <span>{{ model }}</span>
</template>
