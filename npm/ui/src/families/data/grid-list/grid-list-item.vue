<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef } from "vue";

import type { DragSourceProps } from "../../interaction/drag-and-drop/drag-and-drop-controller-types.ts";
import { gridListContext } from "./grid-list-context.ts";

const {
  itemKey,
  index,
  selected = false,
  disabled = false,
  textValue = undefined,
} = defineProps<{
  /**
   * Stable item key (registry key and DOM data hook).
   *
   * @default undefined
   */
  readonly itemKey: string;

  /**
   * Zero-based item index, published as `aria-rowindex` (one-based).
   *
   * @default undefined
   */
  readonly index: number;

  /**
   * Whether the item is selected.
   *
   * @default false
   */
  readonly selected?: boolean;

  /**
   * Whether the item is disabled (focusable, not selectable or draggable).
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Typeahead text; rendered text is used when omitted.
   *
   * @default undefined
   */
  readonly textValue?: string;
}>();

defineSlots<{
  /** Item contents. Receives drag state and handle props. */
  default(props: {
    readonly active: boolean;
    readonly dragging: boolean;
    readonly dragHandleProps: Readonly<DragSourceProps> | null;
  }): unknown;
}>();

const list = gridListContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const registration = list.registry.register({
  key: itemKey,
  value: index,
  element,
  textValue: () => textValue,
  disabled: () => disabled,
});
const sortItem = list.sortable?.registerItem({
  key: itemKey,
  element,
  label: () => textValue ?? element.value?.textContent?.trim() ?? itemKey,
  isDisabled: () => disabled,
});
list.publishHandle(itemKey, sortItem?.itemProps ?? null);
onScopeDispose(() => {
  list.publishHandle(itemKey, null);
  sortItem?.dispose();
  registration.unregister();
});

const active = computed(() => {
  const key = list.registry.activeKey.value ?? list.registry.navigableItems.value[0]?.key ?? null;
  return key === itemKey;
});
const dragging = computed(
  () => (sortItem?.isDragging.value ?? false) || list.sortable?.activeKey.value === itemKey,
);

// The row owns focus (roving tabindex); its single cell is a structural container.
const cellProps = Object.freeze({ role: "gridcell" });

const rowProps = computed(() => ({
  role: "row",
  tabindex: active.value ? 0 : -1,
  onKeydown: (event: KeyboardEvent) => list.onItemKeydown(itemKey, event),
  onClick: (event: MouseEvent) => list.onItemClick(itemKey, event),
  onFocus: () => list.registry.setActiveKey(itemKey),
}));
</script>

<template>
  <div
    v-bind="rowProps"
    :id="`${list.id.value}-${index}`"
    ref="element"
    :aria-rowindex="index + 1"
    :aria-selected="list.selectionMode.value === 'none' ? undefined : selected ? 'true' : 'false'"
    :aria-disabled="disabled ? 'true' : undefined"
    data-vize-ui="grid-list-item"
    part="item"
    :data-key="itemKey"
    :data-selected="selected ? 'true' : undefined"
    :data-active="active ? 'true' : undefined"
    :data-disabled="disabled ? 'true' : undefined"
    :data-dragging="dragging ? 'true' : undefined"
  >
    <div v-bind="cellProps" data-vize-ui="grid-list-cell" part="cell">
      <slot :active :dragging :drag-handle-props="sortItem?.itemProps ?? null" />
    </div>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
