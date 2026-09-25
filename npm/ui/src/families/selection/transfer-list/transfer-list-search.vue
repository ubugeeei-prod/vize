<script setup lang="ts">
import { computed } from "vue";

import { transferListContext } from "./transfer-list-context.ts";
import type { TransferListSide } from "./transfer-list-model.ts";

const {
  side,
  placeholder = undefined,
  ariaLabel = undefined,
} = defineProps<{
  /** Panel this field filters. @default required */
  readonly side: TransferListSide;

  /**
   * Hint text shown while empty.
   *
   * @default undefined
   */
  readonly placeholder?: string;

  /**
   * Accessible name, e.g. "Search available".
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const root = transferListContext.use();
const state = root.side(side);
const handlers = computed(() => ({
  onInput: (event: Event) => {
    if (event.target instanceof HTMLInputElement) root.setQuery(side, event.target.value);
  },
  onKeydown: (event: KeyboardEvent) => {
    if (event.key === "ArrowDown" && !event.isComposing) {
      event.preventDefault();
      root.focusPanel(side);
    } else if (event.key === "Escape" && state.query.value !== "") {
      event.preventDefault();
      root.setQuery(side, "");
    }
  },
}));
</script>

<template>
  <input
    v-bind="handlers"
    type="search"
    :value="state.query.value"
    :placeholder
    :aria-label="ariaLabel"
    :aria-controls="root.panelId(side)"
    :disabled="root.disabled.value"
    autocomplete="off"
    data-vize-ui="transfer-list-search"
    part="search"
    :data-side="side"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
