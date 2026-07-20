<script setup lang="ts">
import { useTemplateRef } from "vue";

const element = useTemplateRef<HTMLSpanElement>("element");

defineExpose({ element });
</script>

<template>
  <span ref="element" data-vize-ui="visually-hidden">
    <slot />
  </span>
</template>

<style scoped>
[data-vize-ui="visually-hidden"] {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip-path: inset(50%);
  white-space: nowrap;
  border: 0;
}
</style>
