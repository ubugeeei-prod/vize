<script setup lang="ts">
defineProps<{
  modelValue?: string;
  tone?: "brand" | "neutral" | "danger";
  count?: number;
  disabled?: boolean;
}>()
</script>

<template>
  <input :value="modelValue" :disabled="disabled" :data-tone="tone" :data-count="count" />
</template>

<art title="Inline Input">
  <variant name="Default">
    <Self model-value="Seed" tone="brand" :disabled="true" />
  </variant>
</art>
