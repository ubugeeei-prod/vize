<script setup lang="ts">
import { computed } from "vue";

import { treeContext, treeItemContext } from "./tree-context.ts";
import type { TreeItemState, TreeLoadState } from "./tree-types.ts";

defineSlots<{
  /** Expand indicator content, for example a chevron. Receives expansion and load state. */
  default(props: {
    /** Whether the owning row has, or may lazily load, children. */
    readonly expandable: boolean;
    /** Whether the owning row is expanded. */
    readonly expanded: boolean;
    /** Lazy children lifecycle for the owning row. */
    readonly loadState: TreeLoadState;
    /** Stable expansion state token. */
    readonly state: TreeItemState;
  }): unknown;
}>();

const context = treeContext.use();
const item = treeItemContext.use();
const state = computed<TreeItemState>(() => {
  if (!item.expandable.value) return "leaf";
  return item.expanded.value ? "expanded" : "collapsed";
});
const slotState = computed(() => ({
  expandable: item.expandable.value,
  expanded: item.expanded.value,
  loadState: item.loadState.value,
  state: state.value,
}));

// The toggle is a pointer affordance inside a treeitem. Keyboard users expand with
// ArrowRight/ArrowLeft on the row, so the toggle stays out of the accessibility tree
// instead of nesting an interactive control inside the treeitem.
const toggleProps = computed<{
  readonly role: "presentation";
  readonly onClick: (event: MouseEvent) => void;
}>(() => ({ role: "presentation", onClick }));

function onClick(event: MouseEvent): void {
  event.stopPropagation();
  if (!item.expandable.value || item.disabled.value) return;
  context.focusKey(item.key.value, { preventScroll: true });
  context.toggleExpanded(item.key.value, event);
}
</script>

<template>
  <span
    v-bind="toggleProps"
    aria-hidden="true"
    data-vize-ui="tree-item-toggle"
    part="toggle"
    :data-state="state"
    :data-disabled="item.disabled.value ? 'true' : undefined"
    :data-load-state="item.loadState.value === 'idle' ? undefined : item.loadState.value"
  >
    <slot v-bind="slotState" />
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
