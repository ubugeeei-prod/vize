<script setup lang="ts">
import { computed, provide, ref, watch } from "vue";

import { segmentedKey } from "./context.ts";
import { nextSegmentIndex } from "./segmented-keyboard.ts";
import type { SegmentedProps } from "./types.ts";

const {
  modelValue = "",
  label,
  disabled = false,
  orientation = "horizontal",
} = defineProps<SegmentedProps>();

const emit = defineEmits<{
  /**
   * Fired when an item is chosen from a click or a roving key.
   *
   * Carries the item value.
   */
  "update:modelValue": [value: string];
}>();

defineSlots<{
  /** Renders `SegmentedItem` options. */
  default(): unknown;
}>();

const selected = ref(modelValue);
const holder: { fallback?: string } = {};
const groupDisabled = computed(() => disabled);
const model = computed(() => selected.value);
const fallbackValue = computed(() => holder.fallback);

watch(
  () => modelValue,
  (next) => {
    selected.value = next;
  },
);

function claim(value: string): void {
  if (holder.fallback === undefined) holder.fallback = value;
}

function choose(value: string): void {
  if (disabled) return;
  selected.value = value;
  emit("update:modelValue", value);
}

provide(segmentedKey, {
  model,
  disabled: groupDisabled,
  fallback: fallbackValue,
  claim,
  choose,
});

function enabledItems(target: EventTarget | null): HTMLButtonElement[] {
  if (!(target instanceof HTMLElement)) return [];
  return [...target.querySelectorAll<HTMLButtonElement>("[data-vize-ui='segmented-item']")].filter(
    (item) => !item.disabled,
  );
}

function onKeydown(event: KeyboardEvent): void {
  if (disabled) return;
  const items = enabledItems(event.currentTarget);
  const origin = event.target instanceof HTMLButtonElement ? event.target : null;
  const current = origin === null ? -1 : items.indexOf(origin);
  const next = nextSegmentIndex(current < 0 ? 0 : current, items.length, event.key);
  if (next === undefined) return;
  const item = items[next];
  const value = item?.dataset.value;
  if (item === undefined || value === undefined) return;
  event.preventDefault();
  choose(value);
  item.focus();
}
</script>

<template>
  <!-- eslint-disable-next-line a11y/interactive-supports-focus -->
  <div
    role="radiogroup"
    data-vize-ui="segmented"
    :aria-label="label"
    :aria-disabled="disabled ? 'true' : undefined"
    :aria-orientation="orientation"
    :data-orientation="orientation"
    @keydown="onKeydown"
  >
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
