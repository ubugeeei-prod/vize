<script setup lang="ts">
import { computed } from "vue";

import { transferListContext } from "./transfer-list-context.ts";
import type { TransferListSide } from "./transfer-list-model.ts";

const { side } = defineProps<{
  /** Panel whose emptiness this message reports. @default required */
  readonly side: TransferListSide;
}>();

defineSlots<{
  /** Message shown while the panel has no visible items. Receives the search text. */
  default?(props: { readonly query: string; readonly filtered: boolean }): unknown;
}>();

const root = transferListContext.use();
const state = root.side(side);
const visible = computed(() => state.visible.value.length === 0);
</script>

<template>
  <div
    role="status"
    data-vize-ui="transfer-list-empty"
    part="empty"
    :data-side="side"
    :hidden="visible ? undefined : true"
  >
    <slot v-if="visible" :query="state.query.value" :filtered="state.all.value.length > 0" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
