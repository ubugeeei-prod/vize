<script setup lang="ts" generic="T">
import { computed, onUnmounted, useTemplateRef, watch } from "vue";

import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { transferListContext, transferListPanelContext } from "./transfer-list-context.ts";
import type { TransferListItemSlotState } from "./transfer-list-types.ts";

const {
  id = undefined,
  value,
  textValue = undefined,
} = defineProps<{
  /**
   * Consumer-owned option id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /** Item value from the panel's `items`. @default required */
  readonly value: T;

  /**
   * Typeahead text; defaults to the root `itemText`.
   *
   * @default undefined
   */
  readonly textValue?: string;
}>();

defineSlots<{
  /** Item content. Receives checked, active, and disabled state. */
  default?(props: TransferListItemSlotState<T>): unknown;
}>();

const root = transferListContext.use();
const panel = transferListPanelContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const optionId = useDeterministicId({ id: () => id, hint: "transfer-list-option" });
const disabled = computed(() => root.isItemDisabled(value));
const checked = computed(() => root.isChecked(panel.side, value));
const active = computed(() => panel.activeKey.value === optionId.value);
const slotState = computed<TransferListItemSlotState<T>>(() => ({
  active: active.value,
  checked: checked.value,
  disabled: disabled.value,
  side: panel.side,
  value,
}));
const interactiveProps = computed(() => ({
  role: "option" as const,
  onClick,
  onDblclick,
  onPointerdown,
}));
let release: (() => void) | null = null;

watch(
  [() => value, optionId],
  () => {
    release?.();
    release = panel.register({
      disabled: () => disabled.value,
      element: () => element.value,
      id: optionId,
      textValue: () => textValue ?? root.textOf(value),
      value,
    });
  },
  { flush: "sync", immediate: true },
);

onUnmounted(() => {
  release?.();
  release = null;
});

function onPointerdown(event: PointerEvent): void {
  if (disabled.value) return;
  event.preventDefault();
  panel.activate(optionId.value);
  panel.focus();
}

function onClick(): void {
  if (disabled.value) return;
  panel.activate(optionId.value);
  root.toggleChecked(panel.side, value);
}

function onDblclick(event: MouseEvent): void {
  if (disabled.value) return;
  root.moveValues(panel.side, [value], event);
}

defineExpose({ checked, element });
</script>

<template>
  <div
    :id="optionId"
    ref="element"
    v-bind="interactiveProps"
    :aria-selected="checked ? 'true' : 'false'"
    :aria-disabled="disabled ? 'true' : undefined"
    data-vize-ui="transfer-list-item"
    part="item"
    :data-state="checked ? 'checked' : 'unchecked'"
    :data-highlighted="active ? 'true' : undefined"
    :data-disabled="disabled ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
