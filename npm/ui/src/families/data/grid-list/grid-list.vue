<script setup lang="ts" generic="Item">
import { computed, nextTick, shallowReactive, useTemplateRef } from "vue";

import type { DragSourceProps } from "../../interaction/drag-and-drop/drag-and-drop-controller-types.ts";
import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { useSortable } from "../../interaction/sortable/sortable.ts";
import type { SortCommitEvent } from "../../interaction/sortable/sortable.ts";
import { useTypeahead } from "../../interaction/typeahead/typeahead.ts";
import { gridListContext } from "./grid-list-context.ts";
import GridListItem from "./grid-list-item.vue";
import type {
  GridListExpose,
  GridListItemSlotProps,
  GridListLayout,
  GridListReorderEvent,
  GridListSelectionMode,
} from "./grid-list-types.ts";

const {
  items,
  getKey = undefined,
  getTextValue = undefined,
  id = undefined,
  selectionMode = "single",
  selection = undefined,
  defaultSelection = [],
  disabledKeys = [],
  layout = "list",
  columns = 1,
  dir = "ltr",
  loop = false,
  reorderable = false,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * Items rendered as rows.
   *
   * @default undefined
   */
  readonly items: readonly Item[];

  /**
   * Stable key per item; defaults to the index.
   *
   * @default undefined
   */
  readonly getKey?: (item: Item, index: number) => string;

  /**
   * Typeahead text per item; defaults to the rendered text.
   *
   * @default undefined
   */
  readonly getTextValue?: (item: Item) => string;

  /**
   * Consumer-owned list id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Selection policy.
   *
   * @default "single"
   */
  readonly selectionMode?: GridListSelectionMode;

  /**
   * Controlled selected keys (`v-model:selection`).
   *
   * @default undefined
   */
  readonly selection?: readonly string[];

  /**
   * Initial uncontrolled selection.
   *
   * @default []
   */
  readonly defaultSelection?: readonly string[];

  /**
   * Keys that stay focusable but cannot be selected, activated, or dragged.
   *
   * @default []
   */
  readonly disabledKeys?: readonly string[];

  /**
   * `list` moves with Up/Down; `grid` also moves with Left/Right and steps Up/Down by `columns`.
   *
   * @default "list"
   */
  readonly layout?: GridListLayout;

  /**
   * Items per visual row in the `grid` layout.
   *
   * @default 1
   */
  readonly columns?: number;

  /**
   * Reading direction for Left/Right in the `grid` layout.
   *
   * @default "ltr"
   */
  readonly dir?: "ltr" | "rtl";

  /**
   * Wrap arrow navigation at both ends.
   *
   * @default false
   */
  readonly loop?: boolean;

  /**
   * Allow pointer and keyboard reordering through the slot's `dragHandleProps`.
   *
   * @default false
   */
  readonly reorderable?: boolean;

  /**
   * Accessible name of the grid.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Ids that label the grid.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

const emit = defineEmits<{
  /** Fired with the new selected keys (supports `v-model:selection`). */
  "update:selection": [value: readonly string[]];
  /** Fired when Enter or a double click activates an enabled item. */
  action: [item: Item, key: string];
  /** Fired with the reordered items (supports `v-model:items`). */
  "update:items": [value: readonly Item[]];
  /** Fired after a reorder commits. */
  reorder: [event: GridListReorderEvent<Item>];
}>();

defineSlots<{
  /** Item contents. */
  item?(props: GridListItemSlotProps<Item>): unknown;
  /** Rendered when `items` is empty. */
  empty?(): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const listId = useDeterministicId({ id: () => id, hint: "grid-list" });
const registry = createCollectionRegistry<string, number>({ disabledBehavior: "focusable" });
const selectionState = useControllableState<readonly string[]>({
  value: () => selection,
  defaultValue: () => defaultSelection,
});
const modeState = computed(() => selectionMode);
const keys = computed(() =>
  items.map((item, index) => (getKey ? getKey(item, index) : String(index))),
);
const disabledSet = computed(() => new Set(disabledKeys));
let anchorKey: string | null = null;

function setSelection(next: readonly string[]): boolean {
  if (!selectionState.set(next)) return false;
  emit("update:selection", next);
  return true;
}

function select(key: string, intent: "range" | "replace" | "toggle"): boolean {
  if (selectionMode === "none" || disabledSet.value.has(key)) return false;
  const current = selectionState.value.value;
  if (selectionMode === "single") {
    return setSelection(current.includes(key) && intent === "toggle" ? [] : [key]);
  }
  if (intent === "range" && anchorKey !== null) {
    const from = keys.value.indexOf(anchorKey);
    const to = keys.value.indexOf(key);
    const range = keys.value
      .slice(Math.min(from, to), Math.max(from, to) + 1)
      .filter((candidate) => !disabledSet.value.has(candidate));
    return setSelection([...new Set([...current, ...range])]);
  }
  anchorKey = key;
  if (intent === "replace") return setSelection([key]);
  return setSelection(
    current.includes(key) ? current.filter((candidate) => candidate !== key) : [...current, key],
  );
}

function focusKey(key: string | null): void {
  if (key === null) return;
  registry.setActiveKey(key);
  void nextTick(() => {
    const target = registry.getItem(key)?.element;
    if (target instanceof HTMLElement) target.focus();
  });
}

const typeahead = useTypeahead({ registry, onMatch: (match) => focusKey(match.key) });

function step(key: string, delta: number): string | null {
  const navigable = registry.navigableItems.value.map((item) => item.key);
  const from = navigable.indexOf(key);
  if (from === -1) return navigable[0] ?? null;
  let to = from + delta;
  if (loop) to = ((to % navigable.length) + navigable.length) % navigable.length;
  return navigable[Math.min(Math.max(to, 0), navigable.length - 1)] ?? null;
}

function onItemKeydown(key: string, event: KeyboardEvent): void {
  // Handle and descendant widgets own their own keys (including drag handles).
  if (event.defaultPrevented || event.target !== event.currentTarget || event.isComposing) return;
  const grid = layout === "grid";
  const rowStep = grid ? Math.max(1, Math.trunc(columns)) : 1;
  const forward = dir === "rtl" ? "ArrowLeft" : "ArrowRight";
  const backward = dir === "rtl" ? "ArrowRight" : "ArrowLeft";
  const modifier = event.ctrlKey || event.metaKey;
  let next: string | null = null;
  if (event.key === "ArrowDown") next = step(key, rowStep);
  else if (event.key === "ArrowUp") next = step(key, -rowStep);
  else if (grid && event.key === forward) next = step(key, 1);
  else if (grid && event.key === backward) next = step(key, -1);
  else if (event.key === "Home") next = registry.getNavigationKey("first");
  else if (event.key === "End") next = registry.getNavigationKey("last");
  else if (event.key === "PageDown") next = step(key, rowStep * 10);
  else if (event.key === "PageUp") next = step(key, -rowStep * 10);
  else if (event.key === " " && typeahead.query.value.length === 0) {
    event.preventDefault();
    select(key, event.shiftKey ? "range" : "toggle");
    return;
  } else if (event.key === "Enter") {
    event.preventDefault();
    const index = keys.value.indexOf(key);
    const item = items[index];
    if (item !== undefined && !disabledSet.value.has(key)) emit("action", item, key);
    return;
  } else if (modifier && (event.key === "a" || event.key === "A")) {
    if (selectionMode !== "multiple") return;
    event.preventDefault();
    setSelection(keys.value.filter((candidate) => !disabledSet.value.has(candidate)));
    return;
  } else {
    typeahead.typeaheadProps.onKeydown(event);
    return;
  }
  event.preventDefault();
  if (next === null) return;
  if (event.shiftKey && selectionMode === "multiple" && next !== key) {
    if (anchorKey === null) anchorKey = key;
    select(next, "range");
  }
  focusKey(next);
}

function onItemClick(key: string, event: MouseEvent): void {
  registry.setActiveKey(key);
  if (event.detail > 1) {
    const item = items[keys.value.indexOf(key)];
    if (item !== undefined && !disabledSet.value.has(key)) emit("action", item, key);
    return;
  }
  const intent = event.shiftKey ? "range" : event.ctrlKey || event.metaKey ? "toggle" : "replace";
  select(key, selectionMode === "multiple" ? intent : "replace");
}

function commitReorder(event: SortCommitEvent): void {
  const moved = items[event.fromIndex];
  if (moved === undefined) return;
  const rest = items.filter((_, index) => index !== event.fromIndex);
  const next = [...rest.slice(0, event.toIndex), moved, ...rest.slice(event.toIndex)];
  emit("update:items", next);
  emit("reorder", {
    item: moved,
    key: event.key,
    fromIndex: event.fromIndex,
    toIndex: event.toIndex,
    items: next,
  });
}

const sortable = reorderable
  ? useSortable({
      orientation: () => (layout === "grid" ? "grid" : "vertical"),
      columns: () => Math.max(1, Math.trunc(columns)),
      direction: () => dir,
      onSortCommit: commitReorder,
    })
  : null;

const handles = shallowReactive(new Map<string, Readonly<DragSourceProps>>());

gridListContext.provide({
  id: listId,
  registry,
  selectionMode: modeState,
  sortable,
  handles,
  publishHandle: (key, props) => {
    if (props) handles.set(key, props);
    else handles.delete(key);
  },
  onItemKeydown,
  onItemClick,
});

function slotProps(item: Item, index: number): GridListItemSlotProps<Item> {
  const key = keys.value[index] ?? String(index);
  const tabStop = registry.activeKey.value ?? registry.navigableItems.value[0]?.key ?? null;
  return {
    item,
    key,
    index,
    selected: selectionState.value.value.includes(key),
    disabled: disabledSet.value.has(key),
    active: tabStop === key,
    dragging: sortable?.activeKey.value === key,
    dragHandleProps: handles.get(key) ?? null,
  };
}

function textProps(item: Item): { readonly textValue?: string } {
  return getTextValue ? { textValue: getTextValue(item) } : {};
}

const emptyRowProps = Object.freeze({ role: "row" });
const emptyCellProps = Object.freeze({ role: "gridcell" });

const exposed = {
  element,
  focus: (key?: string) =>
    focusKey(key ?? registry.activeKey.value ?? registry.getNavigationKey("first")),
  selectAll: () =>
    selectionMode === "multiple" &&
    setSelection(keys.value.filter((candidate) => !disabledSet.value.has(candidate))),
  clearSelection: () => setSelection([]),
} satisfies Omit<GridListExpose, "element"> & { readonly element: typeof element };

defineExpose(exposed);
</script>

<template>
  <div
    :id="listId"
    ref="element"
    role="grid"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-rowcount="items.length"
    :aria-multiselectable="selectionMode === 'multiple' ? 'true' : undefined"
    data-vize-ui="grid-list"
    part="root"
    :data-layout="layout"
    :data-empty="items.length === 0 ? 'true' : undefined"
    :dir
  >
    <GridListItem
      v-for="(item, index) in items"
      :key="keys[index] ?? index"
      :item-key="keys[index] ?? String(index)"
      :index
      :selected="selectionState.value.value.includes(keys[index] ?? '')"
      :disabled="disabledSet.has(keys[index] ?? '')"
      v-bind="textProps(item)"
    >
      <slot name="item" v-bind="slotProps(item, index)">{{ String(item) }}</slot>
    </GridListItem>
    <div
      v-if="items.length === 0"
      v-bind="emptyRowProps"
      data-vize-ui="grid-list-empty"
      part="empty"
    >
      <div v-bind="emptyCellProps"><slot name="empty" /></div>
    </div>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
