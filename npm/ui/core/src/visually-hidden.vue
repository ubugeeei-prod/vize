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
@layer vize.ui {
  [data-vize-ui="visually-hidden"] {
    /* Authored in the native CSS contract: nesting, cascade layers, logical
       properties, and native color functions. The package build down-compiles
       this block to the declared browser floor; see style-pipeline.behavior.md.
       The layer lets consumers order package CSS below their own rules without
       specificity fights. */
    position: absolute;
    inline-size: 1px;
    block-size: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
    border: 0;
    /* Visually inert in every color space; never paints through a consumer
       background. Consumers may repaint the clipped box for debugging through
       the documented custom property. */
    background-color: var(--vize-ui-visually-hidden-background, oklch(0% 0 0 / 0%));

    /* A slotted control that receives focus stays clipped: revealing on focus
       is the visually-hidden-focusable pattern, deliberately not this one.
       :where() keeps the guard at base specificity so consumers stay in charge. */
    &:where(:focus-within) {
      clip-path: inset(50%);
    }
  }
}
</style>
