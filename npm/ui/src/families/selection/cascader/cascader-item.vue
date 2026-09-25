<script setup lang="ts" generic="T">
import { computed } from "vue";

import { cascaderColumnContext, cascaderContext } from "./cascader-context.ts";
import type { CascaderItemSlotState, CascaderItemState } from "./cascader-types.ts";

const { value, ariaLabel = undefined } = defineProps<{
  /** Node rendered by this option; must belong to the column's options. @default required */
  readonly value: T;

  /**
   * Accessible name when the option text is not enough.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Option content. Receives branch, expansion, selection, and loading state. */
  default?(props: CascaderItemSlotState<T>): unknown;
}>();

const context = cascaderContext.use();
const column = cascaderColumnContext.use();
const level = computed(() => column.level.value);
const branch = computed(() => context.isBranch(value));
const expanded = computed(() => branch.value && context.isExpanded(level.value, value));
const loading = computed(() => context.isLoading(value));
const active = computed(() => context.isActive(level.value, value));
const selected = computed(() => context.isSelectedEnd(level.value, value));
const partial = computed(() => !selected.value && context.isPartial(level.value, value));
const disabled = computed(() => context.isDisabled(value));
const state = computed<CascaderItemState>(() => {
  if (selected.value) return "checked";
  return partial.value ? "partial" : "unchecked";
});
const slotState = computed<CascaderItemSlotState<T>>(() => ({
  active: active.value,
  branch: branch.value,
  disabled: disabled.value,
  expanded: expanded.value,
  level: level.value,
  loading: loading.value,
  partial: partial.value,
  selected: selected.value,
  state: state.value,
  value,
}));
const interactiveProps = computed(() => ({
  role: "option" as const,
  onClick: (event: MouseEvent) => context.pressItem(level.value, value, event),
  onPointermove: (event: PointerEvent) => {
    if (event.pointerType !== "touch" && !active.value) context.hoverItem(level.value, value);
  },
}));
</script>

<template>
  <div
    :id="context.optionId(level, value)"
    v-bind="interactiveProps"
    :aria-selected="selected ? 'true' : 'false'"
    :aria-expanded="branch ? (expanded ? 'true' : 'false') : undefined"
    :aria-disabled="disabled ? 'true' : undefined"
    :aria-busy="loading ? 'true' : undefined"
    :aria-label="ariaLabel"
    data-vize-ui="cascader-item"
    part="item"
    :data-state="state"
    :data-branch="branch ? 'true' : undefined"
    :data-highlighted="active ? 'true' : undefined"
    :data-loading="loading ? 'true' : undefined"
    :data-disabled="disabled ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
