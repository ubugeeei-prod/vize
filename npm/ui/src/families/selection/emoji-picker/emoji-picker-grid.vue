<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef } from "vue";

import { gridMoveFromKey } from "../listbox-grid/listbox-grid-model.ts";
import { emojiPickerContext } from "./emoji-picker-context.ts";

const { ariaLabel = "Emoji" } = defineProps<{
  /**
   * Accessible name of the grid.
   *
   * @default "Emoji"
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** One `EmojiPickerCategory` per section from the root slot state. */
  default?(): unknown;
}>();

const context = emojiPickerContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const activeId = computed(() => {
  const index = context.activeIndex.value;
  if (index < 0) return undefined;
  const sections = context.sections.value;
  for (let section = 0; section < sections.length; section++) {
    const items = sections[section]?.items.length ?? 0;
    for (let item = 0; item < items; item++) {
      if (context.virtualIndex(section, item) === index) return context.cellId(section, item);
    }
  }
  return undefined;
});
const interactiveProps = computed(() => ({
  role: "grid" as const,
  tabindex: 0 as const,
  onFocus,
  onKeydown,
}));

function focus(): void {
  element.value?.focus();
}

onScopeDispose(context.registerGrid(focus));

function onFocus(event: FocusEvent): void {
  if (event.target === event.currentTarget && context.activeIndex.value < 0) context.move("first");
}

function onKeydown(event: KeyboardEvent): void {
  if (event.defaultPrevented || event.target !== event.currentTarget) return;
  const move = gridMoveFromKey(event);
  if (move !== null) {
    event.preventDefault();
    context.move(move);
    return;
  }
  if (event.key === "Enter" || event.key === " ") {
    if (context.selectActive(event)) event.preventDefault();
  }
}
</script>

<template>
  <div
    :id="context.gridId.value"
    ref="element"
    v-bind="interactiveProps"
    :aria-label="ariaLabel"
    :aria-rowcount="context.totalRows.value"
    :aria-colcount="context.columns.value"
    :aria-activedescendant="activeId"
    data-vize-ui="emoji-picker-grid"
    part="grid"
    :data-columns="context.columns.value"
    :data-empty="context.empty.value ? 'true' : undefined"
  >
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
