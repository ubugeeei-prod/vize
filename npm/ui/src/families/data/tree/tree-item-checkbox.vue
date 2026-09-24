<script setup lang="ts">
import { computed } from "vue";

import { treeContext, treeItemContext } from "./tree-context.ts";
import type { TreeCheckedState } from "./tree-types.ts";

defineSlots<{
  /** Checkbox indicator content. Receives the tri-state value of the owning row. */
  default(props: {
    /** Tri-state checkbox value of the owning row. */
    readonly checked: TreeCheckedState;
    /** Whether the owning row or the tree is disabled. */
    readonly disabled: boolean;
  }): unknown;
}>();

const context = treeContext.use();
const item = treeItemContext.use();
const slotState = computed(() => ({
  checked: item.checked.value,
  disabled: item.disabled.value,
}));

// The checkbox state is announced through `aria-checked` on the treeitem itself
// (enable `checkable` on TreeRoot) and toggled with Space, so this indicator is a
// presentational pointer affordance rather than a nested interactive control.
const checkboxProps = computed<{
  readonly role: "presentation";
  readonly onClick: (event: MouseEvent) => void;
}>(() => ({ role: "presentation", onClick }));

function onClick(event: MouseEvent): void {
  event.stopPropagation();
  if (item.disabled.value) return;
  context.focusKey(item.key.value, { preventScroll: true });
  context.toggleChecked(item.key.value, event);
}
</script>

<template>
  <span
    v-bind="checkboxProps"
    aria-hidden="true"
    data-vize-ui="tree-item-checkbox"
    part="checkbox"
    :data-state="item.checked.value"
    :data-disabled="item.disabled.value ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
