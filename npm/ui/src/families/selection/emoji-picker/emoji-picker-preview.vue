<script setup lang="ts">
import { computed } from "vue";

import { emojiPickerContext } from "./emoji-picker-context.ts";
import type { EmojiPickerPreviewSlotState } from "./emoji-picker-types.ts";

defineSlots<{
  /** Preview of the highlighted emoji. Defaults to its glyph and name. */
  default?(props: EmojiPickerPreviewSlotState): unknown;
}>();

const context = emojiPickerContext.use();
const glyph = computed<string>(() => slotState.value.glyph);
const name = computed<string>(() => slotState.value.name);
const empty = computed(() => context.activeItem.value === undefined);
const slotState = computed<EmojiPickerPreviewSlotState>(() => {
  const item = context.activeItem.value;
  return item === undefined
    ? { glyph: "", item: undefined, name: "" }
    : { glyph: context.glyphOf(item), item, name: context.nameOf(item) };
});
</script>

<template>
  <div
    aria-live="polite"
    data-vize-ui="emoji-picker-preview"
    part="preview"
    :data-empty="empty ? 'true' : undefined"
  >
    <slot v-bind="slotState">{{ glyph }} {{ name }}</slot>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
