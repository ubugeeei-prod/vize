<script setup lang="ts">
const { level = 1 } = defineProps<{
  /**
   * Nesting depth exposed to styles; nested lists live inside a TocItem.
   *
   * @default 1
   */
  readonly level?: number;
}>();

defineSlots<{
  /** TocItem children. */
  default(props: {
    /** Nesting depth of this list. */
    readonly level: number;
  }): unknown;
}>();
</script>

<template>
  <ol data-vize-ui="toc-list" part="list" :data-level="level">
    <slot :level="level" />
  </ol>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
