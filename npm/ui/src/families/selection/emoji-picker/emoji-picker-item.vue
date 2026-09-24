<script setup lang="ts" generic="T">
import { computed } from "vue";

import { emojiPickerCategoryContext, emojiPickerContext } from "./emoji-picker-context.ts";
import type { EmojiPickerItemSlotState } from "./emoji-picker-types.ts";

const { item, index } = defineProps<{
  /** Emoji item rendered by this cell. @default required */
  readonly item: T;

  /** Item index inside its section, from the category slot. @default required */
  readonly index: number;
}>();

defineSlots<{
  /** Cell content; defaults to the glyph. Receives glyph, name, and highlight state. */
  default?(props: EmojiPickerItemSlotState<T>): unknown;
}>();

const context = emojiPickerContext.use();
const category = emojiPickerCategoryContext.use();
const cellId = computed(() => context.cellId(Math.max(0, category.section.value), index));
const virtualIndex = computed(() => context.virtualIndex(category.section.value, index));
const active = computed(() => context.activeIndex.value === virtualIndex.value);
const glyph = computed(() => context.glyphOf(item));
const name = computed(() => context.nameOf(item));
const slotState = computed<EmojiPickerItemSlotState<T>>(() => ({
  active: active.value,
  glyph: glyph.value,
  item,
  name: name.value,
}));
const interactiveProps = computed(() => ({
  role: "gridcell" as const,
  onClick: (event: MouseEvent) => {
    context.setActive(virtualIndex.value);
    context.select(item, event);
  },
  onPointerdown: (event: PointerEvent) => {
    event.preventDefault();
    context.setActive(virtualIndex.value);
    context.focusGrid();
  },
  onPointermove: () => {
    if (!active.value) context.setActive(virtualIndex.value);
  },
}));
</script>

<template>
  <div
    :id="cellId"
    v-bind="interactiveProps"
    :aria-label="name"
    data-vize-ui="emoji-picker-item"
    part="item"
    :data-highlighted="active ? 'true' : undefined"
  >
    <slot v-bind="slotState">{{ glyph }}</slot>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
