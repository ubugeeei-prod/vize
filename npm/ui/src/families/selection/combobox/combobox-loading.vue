<script setup lang="ts">
import { computed } from "vue";

import { comboboxContext } from "./combobox-context.ts";

defineSlots<{
  /** Content shown while `loadItems` is pending. Receives the query being loaded. */
  default(props: { readonly query: string }): unknown;
}>();

const context = comboboxContext.use();
const visible = computed(() => context.status.value === "loading");
</script>

<template>
  <div
    role="presentation"
    aria-busy="true"
    data-vize-ui="combobox-loading"
    part="loading"
    :hidden="visible ? undefined : true"
  >
    <slot v-if="visible" :query="context.query.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
