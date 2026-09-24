<script setup lang="ts" generic="T, K extends TreeKey">
import { computed, onScopeDispose, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { treeContext, treeItemContext } from "./tree-context.ts";
import type {
  TreeCheckedState,
  TreeDropPosition,
  TreeFlatNode,
  TreeItemExpose,
  TreeItemSlotState,
  TreeItemState,
  TreeKey,
  TreeLoadState,
  TreeReorderItemRegistration,
} from "./tree-types.ts";

const {
  item,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /** Visible row from the TreeRoot slot `items`. @default required */
  readonly item: TreeFlatNode<T, K>;

  /**
   * Accessible name when the row text does not supply one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the row.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the row.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** Row content. Receives the typed node plus expansion, selection, and checkbox state. */
  default?(props: TreeItemSlotState<T, K>): unknown;
}>();

const context = treeContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const key = computed<K>(() => item.key);
const itemId = computed(() => context.getItemId(item.key));
const itemDisabled = computed(() => context.isDisabled(item.key));
const selected = computed(() => context.isSelected(item.key));
const active = computed(() => context.activeKey.value === item.key);
const checked = computed<TreeCheckedState>(() => context.getCheckedState(item.key));
const loadState = computed<TreeLoadState>(() => context.getLoadState(item.key));
const expandable = computed(() => item.expandable);
const expanded = computed(() => item.expanded);
const itemState = computed<TreeItemState>(() => {
  if (!item.expandable) return "leaf";
  return item.expanded ? "expanded" : "collapsed";
});
const dropPosition = computed<TreeDropPosition | null>(() => context.getDropPosition(item.key));
const ariaChecked = computed(() => {
  if (!context.checkable.value) return undefined;
  return checked.value === "checked" ? "true" : checked.value === "mixed" ? "mixed" : "false";
});
const ariaSelected = computed(() => {
  if (context.selectionMode.value === "none") return undefined;
  return selected.value ? "true" : "false";
});
const slotState = computed<TreeItemSlotState<T, K>>(() => ({
  active: active.value,
  checked: checked.value,
  disabled: itemDisabled.value,
  expandable: item.expandable,
  expanded: item.expanded,
  key: item.key,
  level: item.level,
  loadState: loadState.value,
  node: item.node,
  selected: selected.value,
  state: itemState.value,
}));

treeItemContext.provide({
  checked,
  disabled: itemDisabled,
  expandable,
  expanded,
  key,
  loadState,
});

watch(
  key,
  (next, _previous, onCleanup) => onCleanup(context.registerElement({ key: next, element })),
  {
    flush: "sync",
    immediate: true,
  },
);

watch(
  [element, () => item.index],
  ([target, position]) => {
    if (context.virtualized.value) context.measureElement(target, position);
  },
  { flush: "post" },
);

const reorderRegistration = shallowRef<TreeReorderItemRegistration | null>(null);
watch(
  [key, context.reorderable],
  ([next, reorderable], _previous, onCleanup) => {
    if (!reorderable) return;
    const registration = context.registerReorderItem(
      next,
      () => element.value,
      () => element.value?.textContent?.trim() || String(next),
    );
    reorderRegistration.value = registration;
    onCleanup(() => {
      registration?.dispose();
      if (reorderRegistration.value === registration) reorderRegistration.value = null;
    });
  },
  { flush: "sync", immediate: true },
);
onScopeDispose(() => {
  reorderRegistration.value?.dispose();
  reorderRegistration.value = null;
});
const dragging = computed(() => reorderRegistration.value?.isDragging.value === true);

function onClick(event: MouseEvent): void {
  context.onItemClick(item.key, event);
}

function onFocus(event: FocusEvent): void {
  if (event.target === element.value) context.onItemFocus(item.key);
}

function onPointerdown(event: PointerEvent): void {
  reorderRegistration.value?.pointerProps.onPointerdown(event);
}

function onMousedown(event: MouseEvent): void {
  reorderRegistration.value?.pointerProps.onMousedown(event);
}

function onTouchstart(event: TouchEvent): void {
  reorderRegistration.value?.pointerProps.onTouchstart(event);
}

function onDragstart(event: DragEvent): void {
  reorderRegistration.value?.pointerProps.onDragstart(event);
}

const rowProps = computed<{
  readonly role: "treeitem";
  readonly onClick: (event: MouseEvent) => void;
  readonly onDragstart: (event: DragEvent) => void;
  readonly onFocus: (event: FocusEvent) => void;
  readonly onMousedown: (event: MouseEvent) => void;
  readonly onPointerdown: (event: PointerEvent) => void;
  readonly onTouchstart: (event: TouchEvent) => void;
}>(() => ({
  role: "treeitem",
  onClick,
  onDragstart,
  onFocus,
  onMousedown,
  onPointerdown,
  onTouchstart,
}));

function focus(options?: FocusOptions): void {
  context.focusKey(item.key, options);
}

type TreeItemSetupExpose = Omit<
  TreeItemExpose<T, K>,
  keyof TreeItemSlotState<T, K> | "element" | "id"
> & {
  readonly active: ComputedRef<boolean>;
  readonly checked: ComputedRef<TreeCheckedState>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly expandable: ComputedRef<boolean>;
  readonly expanded: ComputedRef<boolean>;
  readonly id: ComputedRef<string>;
  readonly key: ComputedRef<K>;
  readonly level: ComputedRef<number>;
  readonly loadState: ComputedRef<TreeLoadState>;
  readonly node: ComputedRef<T>;
  readonly selected: ComputedRef<boolean>;
  readonly state: ComputedRef<TreeItemState>;
};

const exposed = {
  active,
  checked,
  disabled: itemDisabled,
  element,
  expandable,
  expanded,
  focus,
  id: itemId,
  key,
  level: computed(() => item.level),
  loadState,
  node: computed(() => item.node),
  selected,
  state: itemState,
} satisfies TreeItemSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    v-bind="rowProps"
    :id="itemId"
    ref="element"
    :tabindex="active && !context.disabled.value ? 0 : -1"
    :aria-level="item.level"
    :aria-setsize="item.setsize"
    :aria-posinset="item.posinset"
    :aria-expanded="item.expandable ? (item.expanded ? 'true' : 'false') : undefined"
    :aria-selected="ariaSelected"
    :aria-checked="ariaChecked"
    :aria-disabled="itemDisabled ? 'true' : undefined"
    :aria-busy="loadState === 'loading' ? 'true' : undefined"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    data-vize-ui="tree-item"
    part="item"
    :data-state="itemState"
    :data-index="item.index"
    :data-level="item.level"
    :data-selected="selected ? 'true' : undefined"
    :data-active="active ? 'true' : undefined"
    :data-disabled="itemDisabled ? 'true' : undefined"
    :data-checked="context.checkable.value ? checked : undefined"
    :data-load-state="loadState === 'idle' ? undefined : loadState"
    :data-dragging="dragging ? 'true' : undefined"
    :data-drop-position="dropPosition ?? undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
