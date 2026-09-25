<script setup lang="ts">
import { computed } from "vue";

import { transferListContext } from "./transfer-list-context.ts";
import type { TransferListAction } from "./transfer-list-model.ts";

const { action, ariaLabel = undefined } = defineProps<{
  /** Move checked or all visible items in one direction. @default required */
  readonly action: TransferListAction;

  /**
   * Accessible name; defaults to an English description of the action.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Button content such as an arrow icon. Receives availability. */
  default?(props: { readonly enabled: boolean }): unknown;
}>();

const defaultLabels: Readonly<Record<TransferListAction, string>> = {
  "move-all-to-source": "Remove all",
  "move-all-to-target": "Add all",
  "move-selected-to-source": "Remove selected",
  "move-selected-to-target": "Add selected",
};

const root = transferListContext.use();
const enabled = computed(() => root.canRun(action));
const label = computed(() => ariaLabel ?? defaultLabels[action]);
const handlers = computed(() => ({
  onClick: (event: MouseEvent) => {
    if (enabled.value) root.run(action, event);
  },
}));
</script>

<template>
  <button
    v-bind="handlers"
    type="button"
    :disabled="!enabled"
    :aria-label="label"
    :aria-controls="`${root.panelId('source')} ${root.panelId('target')}`"
    data-vize-ui="transfer-list-action"
    part="action"
    :data-action="action"
  >
    <slot :enabled="enabled" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
