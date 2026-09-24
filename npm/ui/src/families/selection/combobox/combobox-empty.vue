<script setup lang="ts">
import { computed } from "vue";

import { comboboxContext } from "./combobox-context.ts";

defineSlots<{
  /** Message shown when no option is visible and nothing is loading. Receives the query. */
  default(props: { readonly query: string }): unknown;
}>();

const context = comboboxContext.use();
const visible = computed(() => context.empty.value && context.status.value !== "loading");
</script>

<template>
  <div
    role="presentation"
    data-vize-ui="combobox-empty"
    part="empty"
    :hidden="visible ? undefined : true"
  >
    <slot v-if="visible" :query="context.query.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
