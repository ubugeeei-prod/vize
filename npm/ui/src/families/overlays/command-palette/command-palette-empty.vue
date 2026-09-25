<script setup lang="ts">
import { computed } from "vue";

import { commandPaletteContext } from "./command-palette-context.ts";

defineSlots<{
  /** Fallback shown when no item matches and nothing is loading. Receives the search. */
  default?(props: { readonly search: string }): unknown;
}>();

const context = commandPaletteContext.use();
const visible = computed(() => !context.loading.value && context.resultCount.value === 0);
</script>

<template>
  <div
    role="presentation"
    :hidden="visible ? undefined : true"
    data-vize-ui="command-palette-empty"
    part="empty"
  >
    <slot :search="context.search.value">No results</slot>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
