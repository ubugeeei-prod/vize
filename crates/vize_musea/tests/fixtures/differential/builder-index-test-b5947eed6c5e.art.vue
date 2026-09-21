<script setup lang="ts">
defineProps<{
  tone?: "brand" | "neutral";
  disabled?: boolean;
}>()
</script>

<template>
  <button :disabled="disabled">{{ tone }}</button>
</template>

<art title="Inline Button">
  <variant name="Default">
    <Self tone="brand" :disabled="false" />
  </variant>
</art>
