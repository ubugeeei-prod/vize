<script setup lang="ts">
import { computed } from "vue";

import { emojiPickerContext } from "./emoji-picker-context.ts";

const { placeholder = undefined, ariaLabel = "Search emoji" } = defineProps<{
  /**
   * Hint text shown while empty.
   *
   * @default undefined
   */
  readonly placeholder?: string;

  /**
   * Accessible name of the search field.
   *
   * @default "Search emoji"
   */
  readonly ariaLabel?: string;
}>();

const context = emojiPickerContext.use();
const handlers = computed(() => ({
  onInput: (event: Event) => {
    if (event.target instanceof HTMLInputElement) context.setSearch(event.target.value);
  },
  onKeydown: (event: KeyboardEvent) => {
    if (event.isComposing) return;
    if (event.key === "ArrowDown") {
      event.preventDefault();
      if (context.activeIndex.value < 0) context.move("first");
      context.focusGrid();
    } else if (event.key === "Enter") {
      if (context.selectActive(event)) event.preventDefault();
    } else if (event.key === "Escape" && context.search.value !== "") {
      event.preventDefault();
      context.setSearch("");
    }
  },
}));
</script>

<template>
  <input
    v-bind="handlers"
    type="search"
    :value="context.search.value"
    :placeholder
    :aria-label="ariaLabel"
    :aria-controls="context.gridId.value"
    autocomplete="off"
    data-vize-ui="emoji-picker-search"
    part="search"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
