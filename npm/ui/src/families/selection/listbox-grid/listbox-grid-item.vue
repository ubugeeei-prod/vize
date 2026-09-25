<script setup lang="ts" generic="T">
import { computed, onUnmounted, useTemplateRef, watch } from "vue";

import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { listboxGridContext } from "./listbox-grid-context.ts";
import type { ListboxGridItemSlotState, ListboxGridItemState } from "./listbox-grid-types.ts";

const {
  id = undefined,
  value,
  disabled = false,
  textValue = undefined,
  ariaLabel = undefined,
} = defineProps<{
  /**
   * Consumer-owned option id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /** Option value compared with the selection through `by`. @default required */
  readonly value: T;

  /**
   * Disable this option.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Explicit typeahead text; icon and color swatches usually need one.
   *
   * @default undefined
   */
  readonly textValue?: string;

  /**
   * Accessible name, required when the option renders only an icon or swatch.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Option content. Receives position, selection, and highlight state. */
  default?(props: ListboxGridItemSlotState<T>): unknown;
}>();

const context = listboxGridContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const optionId = useDeterministicId({ id: () => id, hint: "listbox-grid-option" });
const itemDisabled = computed(() => context.disabled.value || disabled);
const selected = computed(() => context.isSelected(value));
const active = computed(() => context.activeId.value === optionId.value);
const index = computed(() => context.indexOf(optionId.value));
const state = computed<ListboxGridItemState>(() => {
  if (itemDisabled.value) return "disabled";
  return selected.value ? "checked" : "unchecked";
});
const slotState = computed<ListboxGridItemSlotState<T>>(() => ({
  active: active.value,
  column: Math.max(0, index.value) % context.columns.value,
  disabled: itemDisabled.value,
  index: index.value,
  row: Math.floor(Math.max(0, index.value) / context.columns.value),
  selected: selected.value,
  state: state.value,
  value,
}));
const interactiveProps = computed(() => ({
  role: "option" as const,
  onClick,
  onPointerdown,
}));
let registration: CollectionRegistration<string> | null = null;

watch(
  [() => value, optionId],
  () => {
    registration?.unregister();
    registration = context.register({
      disabled: itemDisabled,
      element,
      id: optionId,
      textValue: () => textValue,
      value,
    });
  },
  { flush: "sync", immediate: true },
);

onUnmounted(() => {
  registration?.unregister();
  registration = null;
});

function onPointerdown(event: PointerEvent): void {
  if (itemDisabled.value) return;
  event.preventDefault();
  context.activate(optionId.value);
  context.focus();
}

function onClick(event: MouseEvent): void {
  if (itemDisabled.value) return;
  context.activate(optionId.value);
  context.choose(value, event);
}

defineExpose({ element, selected, active });
</script>

<template>
  <div
    :id="optionId"
    ref="element"
    v-bind="interactiveProps"
    :aria-selected="selected ? 'true' : 'false'"
    :aria-disabled="itemDisabled ? 'true' : undefined"
    :aria-label="ariaLabel"
    data-vize-ui="listbox-grid-item"
    part="item"
    :data-state="state"
    :data-highlighted="active ? 'true' : undefined"
    :data-disabled="itemDisabled ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
