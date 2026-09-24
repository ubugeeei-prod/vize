<script setup lang="ts" generic="T, K extends TreeKey">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  onScopeDispose,
  shallowReactive,
  shallowRef,
  useTemplateRef,
  watch,
} from "vue";
import type { ComputedRef } from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { useTypeahead } from "../../interaction/typeahead/typeahead.ts";
import { treeContext } from "./tree-context.ts";
import type { TreeContextValue, TreeItemElementInput } from "./tree-context.ts";
import {
  deriveTreeCheckedStates,
  flattenVisibleTree,
  getTreeKeyIdSegment,
  indexTree,
  isTreeDescendant,
  toggleTreeChecked,
  treeKeysEqual,
} from "./tree-model.ts";
import type { TreeIndexEntry } from "./tree-model.ts";
import type {
  TreeCheckedState,
  TreeCheckPropagation,
  TreeDirection,
  TreeDropPosition,
  TreeFlatNode,
  TreeKey,
  TreeLoadContext,
  TreeLoadState,
  TreeReorderController,
  TreeReorderItemRegistration,
  TreeRootExpose,
  TreeSelectionMode,
  TreeSlotState,
  TreeState,
  TreeVirtualizer,
} from "./tree-types.ts";

const {
  id = undefined,
  items,
  getKey,
  getChildren = undefined,
  hasChildren = undefined,
  loadChildren = undefined,
  getTextValue = undefined,
  isDisabled = undefined,
  expanded = undefined,
  defaultExpanded = undefined,
  selected = undefined,
  defaultSelected = undefined,
  checked = undefined,
  defaultChecked = undefined,
  selectionMode = "single",
  selectionFollowsFocus = false,
  expandOnClick = false,
  checkable = false,
  checkPropagation = "cascade",
  disabled = false,
  dir = "ltr",
  typeahead = true,
  typeaheadTimeout = 500,
  virtualizer = undefined,
  reorder = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /**
   * Consumer-owned tree base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /** Root nodes in display order. @default required */
  readonly items: readonly T[];

  /** Resolve a node's stable key. Keys must be unique across the whole tree. @default required */
  readonly getKey: (node: T) => K;

  /**
   * Resolve a node's children. `null` and `undefined` mean "no children yet", which
   * lets `loadChildren` fetch them lazily; an empty array marks a leaf.
   *
   * @default undefined
   */
  readonly getChildren?: (node: T) => readonly T[] | null | undefined;

  /**
   * Whether a node without resolved children may lazily load some. Only consulted with `loadChildren`.
   *
   * @default () => true
   */
  readonly hasChildren?: (node: T) => boolean;

  /**
   * Asynchronously load a node's children the first time it expands.
   *
   * @default undefined
   */
  readonly loadChildren?: (node: T, context: TreeLoadContext) => Promise<readonly T[]>;

  /**
   * Text used by typeahead. `undefined` reads the rendered row text, so virtualized
   * trees should provide it to reach rows outside the window.
   *
   * @default undefined
   */
  readonly getTextValue?: (node: T) => string;

  /**
   * Disable individual nodes. Disabled rows stay focusable but cannot expand, select, or check.
   *
   * @default undefined
   */
  readonly isDisabled?: (node: T) => boolean;

  /**
   * Controlled expanded keys (`v-model:expanded`). `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly expanded?: readonly K[];

  /**
   * Initially expanded keys for uncontrolled use.
   *
   * @default []
   */
  readonly defaultExpanded?: readonly K[];

  /**
   * Controlled selected keys (`v-model:selected`). Single mode keeps at most one key.
   *
   * @default undefined
   */
  readonly selected?: readonly K[];

  /**
   * Initially selected keys for uncontrolled use.
   *
   * @default []
   */
  readonly defaultSelected?: readonly K[];

  /**
   * Controlled checked keys (`v-model:checked`), normalized by `checkPropagation`.
   *
   * @default undefined
   */
  readonly checked?: readonly K[];

  /**
   * Initially checked keys for uncontrolled use. A cascading parent key checks its subtree.
   *
   * @default []
   */
  readonly defaultChecked?: readonly K[];

  /**
   * Selection model: none, one key, or a set of keys.
   *
   * @default "single"
   */
  readonly selectionMode?: TreeSelectionMode;

  /**
   * Whether arrow-key focus also selects in single selection mode.
   *
   * @default false
   */
  readonly selectionFollowsFocus?: boolean;

  /**
   * Whether clicking an expandable row also toggles it.
   *
   * @default false
   */
  readonly expandOnClick?: boolean;

  /**
   * Expose `aria-checked` on rows and make Space toggle checkboxes instead of selection.
   *
   * @default false
   */
  readonly checkable?: boolean;

  /**
   * How checking a node affects its descendants and ancestors.
   *
   * @default "cascade"
   */
  readonly checkPropagation?: TreeCheckPropagation;

  /**
   * Disable the whole tree and remove it from sequential focus order.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Reading direction used to map ArrowRight and ArrowLeft to expand and collapse.
   *
   * @default "ltr"
   */
  readonly dir?: TreeDirection;

  /**
   * Move focus to the next row whose text starts with typed characters.
   *
   * @default true
   */
  readonly typeahead?: boolean;

  /**
   * Idle time in milliseconds before typeahead starts a new query.
   *
   * @default 500
   */
  readonly typeaheadTimeout?: number;

  /**
   * Windowing adapter from `useTreeVirtualizer`. The tree element becomes the scroll viewport.
   *
   * @default undefined
   */
  readonly virtualizer?: TreeVirtualizer;

  /**
   * Drag and `Alt+Arrow` reorder adapter from `useTreeReorder`.
   *
   * @default undefined
   */
  readonly reorder?: TreeReorderController<K>;

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the tree.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the tree.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

const emit = defineEmits<{
  /** Fired with the next expanded keys whenever expansion changes. */
  "update:expanded": [keys: readonly K[]];

  /** Fired with the next selected keys whenever selection changes. */
  "update:selected": [keys: readonly K[]];

  /** Fired with the next normalized checked keys whenever a checkbox changes. */
  "update:checked": [keys: readonly K[]];

  /** Fired when Enter or a double click activates a row, for example to open a file. */
  action: [key: K, node: T, nativeEvent: Event];

  /** Fired after `loadChildren` resolves for a node. */
  load: [key: K, children: readonly T[]];

  /** Fired when `loadChildren` rejects for a node. */
  loadError: [key: K, error: unknown];
}>();

defineSlots<{
  /** TreeItem rows. Receives rendered rows, every visible row, and selection state. */
  default?(props: TreeSlotState<T, K>): unknown;
}>();

const noKeys: readonly K[] = Object.freeze([]);
const element = useTemplateRef<HTMLDivElement>("element");
const baseId = useDeterministicId({ id: () => id, hint: "tree" });
const disabledState = computed(() => disabled);
const checkableState = computed(() => checkable);
const selectionModeState = computed(() => selectionMode);
const dirState = computed(() => dir);
const cascade = computed(() => checkPropagation === "cascade");
const loaded = shallowReactive(new Map<TreeKey, readonly T[]>());
const loadStates = shallowReactive(new Map<TreeKey, TreeLoadState>());
const loadControllers = new Map<TreeKey, AbortController>();
const elements = shallowReactive(new Map<TreeKey, HTMLDivElement>());
const anchorKey = shallowRef<TreeKey | null>(null);

const expandedState = useControllableState<readonly K[]>({
  value: () => expanded,
  defaultValue: () => defaultExpanded ?? noKeys,
  equals: treeKeysEqual,
  onChange: (value) => emit("update:expanded", value),
});
const selectedState = useControllableState<readonly K[]>({
  value: () => selected,
  defaultValue: () => defaultSelected ?? noKeys,
  equals: treeKeysEqual,
  onChange: (value) => emit("update:selected", value),
});
const checkedState = useControllableState<readonly K[]>({
  value: () => checked,
  defaultValue: () => defaultChecked ?? noKeys,
  equals: treeKeysEqual,
  onChange: (value) => emit("update:checked", value),
});

function resolveChildren(node: T, key: K): readonly T[] | null {
  const explicit = getChildren?.(node);
  if (explicit !== null && explicit !== undefined) return explicit;
  return loaded.get(key) ?? null;
}

const index = computed(() => indexTree(items, { getKey, resolveChildren }));
const expandedSet = computed(() => new Set<TreeKey>(expandedState.value.value));
const selectedSet = computed(() => new Set<TreeKey>(selectedState.value.value));
const checkedSet = computed(() => new Set<TreeKey>(checkedState.value.value));
const checkedStates = computed(() =>
  deriveTreeCheckedStates(index.value, checkedSet.value, cascade.value),
);

function needsLoad(entry: TreeIndexEntry<T, K>): boolean {
  if (loadChildren === undefined || entry.childKeys !== null) return false;
  if (loadStates.get(entry.key) === "loaded") return false;
  return hasChildren?.(entry.node) ?? true;
}

function isExpandable(entry: TreeIndexEntry<T, K>): boolean {
  if (entry.childKeys !== null) return entry.childKeys.length > 0;
  return needsLoad(entry);
}

const visibleItems = computed(() =>
  flattenVisibleTree(index.value, expandedSet.value, isExpandable),
);
const rowByKey = computed(
  () => new Map<TreeKey, TreeFlatNode<T, K>>(visibleItems.value.map((row) => [row.key, row])),
);

function isKeyDisabled(key: TreeKey): boolean {
  if (disabledState.value) return true;
  const entry = index.value.entries.get(key);
  return entry !== undefined && isDisabled !== undefined && isDisabled(entry.node);
}

// Every visible row joins a collection registry with an explicit order, so typeahead
// and roving focus work for virtualized rows that are not mounted.
const registry = createCollectionRegistry<K, K>({ disabledBehavior: "focusable" });
const registrations = new Map<TreeKey, CollectionRegistration<K>>();
watch(
  visibleItems,
  (rows) => {
    const live = new Set<TreeKey>();
    for (const row of rows) {
      live.add(row.key);
      if (registrations.has(row.key)) continue;
      const key = row.key;
      registrations.set(
        key,
        registry.register({
          key,
          value: key,
          element: () => elements.get(key) ?? null,
          textValue: () => {
            const node = index.value.entries.get(key)?.node;
            return node === undefined || getTextValue === undefined ? null : getTextValue(node);
          },
          order: () => rowByKey.value.get(key)?.index,
        }),
      );
    }
    for (const [key, registration] of registrations) {
      if (live.has(key)) continue;
      registration.unregister();
      registrations.delete(key);
    }
  },
  { flush: "sync", immediate: true },
);

const activeKey = computed<K | null>(() => {
  const current = registry.activeKey.value;
  if (current !== null && rowByKey.value.has(current)) return current;
  const firstSelected = visibleItems.value.find((row) => selectedSet.value.has(row.key));
  return firstSelected?.key ?? visibleItems.value[0]?.key ?? null;
});

const typeaheadController = useTypeahead({
  registry,
  isDisabled: () => !typeahead || disabledState.value,
  timeout: () => typeaheadTimeout,
  onMatch: (match) => {
    focusKey(match.key);
  },
});

if (virtualizer !== undefined) {
  virtualizer.connect({
    count: () => visibleItems.value.length,
    getKey: (position) => visibleItems.value[position]?.key ?? position,
  });
}

const renderedItems = computed<readonly TreeFlatNode<T, K>[]>(() => {
  if (virtualizer === undefined) return visibleItems.value;
  const rows: TreeFlatNode<T, K>[] = [];
  for (const item of virtualizer.virtualItems.value) {
    const row = visibleItems.value[item.index];
    if (row !== undefined) rows.push(row);
  }
  return rows;
});
const activeRendered = computed(() =>
  renderedItems.value.some((row) => activeKey.value !== null && row.key === activeKey.value),
);
const rootTabindex = computed<0 | undefined>(() => {
  if (disabledState.value || visibleItems.value.length === 0) return undefined;
  return activeRendered.value ? undefined : 0;
});
const state = computed<TreeState>(() => {
  if (disabledState.value) return "disabled";
  return visibleItems.value.length === 0 ? "empty" : "ready";
});
const slotState = computed<TreeSlotState<T, K>>(() => ({
  activeKey: activeKey.value,
  checked: checkedState.value.value,
  disabled: disabledState.value,
  expanded: expandedState.value.value,
  items: renderedItems.value,
  selected: selectedState.value.value,
  selectionMode: selectionModeState.value,
  state: state.value,
  visibleItems: visibleItems.value,
}));

function getItemId(key: TreeKey): string {
  return deriveDeterministicId(baseId.value, `item-${getTreeKeyIdSegment(key)}`);
}

function focusWithin(): boolean {
  return element.value?.contains(element.value.ownerDocument.activeElement) ?? false;
}

function currentActiveKey(): K | null {
  return activeKey.value;
}

function currentRows(): readonly TreeFlatNode<T, K>[] {
  return visibleItems.value;
}

function focusKey(key: TreeKey, options?: FocusOptions): boolean {
  const row = rowByKey.value.get(key);
  if (row === undefined || disabledState.value) return false;
  registry.setActiveKey(row.key);
  virtualizer?.scrollToIndex(row.index);
  const target = elements.get(row.key);
  if (target !== undefined) {
    target.focus(options);
    return true;
  }
  void nextTick(() => elements.get(row.key)?.focus(options));
  return true;
}

function focus(options?: FocusOptions): void {
  const key = currentActiveKey();
  if (key !== null) focusKey(key, options);
}

function setExpandedKeys(next: readonly K[]): boolean {
  return expandedState.set(next);
}

async function load(entry: TreeIndexEntry<T, K>): Promise<boolean> {
  if (loadChildren === undefined) return false;
  loadControllers.get(entry.key)?.abort();
  const controller = new AbortController();
  loadControllers.set(entry.key, controller);
  loadStates.set(entry.key, "loading");
  try {
    const children = await loadChildren(entry.node, { signal: controller.signal });
    if (controller.signal.aborted) return false;
    loaded.set(entry.key, children);
    loadStates.set(entry.key, "loaded");
    emit("load", entry.key, children);
    return true;
  } catch (error) {
    if (controller.signal.aborted) return false;
    loadStates.set(entry.key, "error");
    emit("loadError", entry.key, error);
    return false;
  } finally {
    if (loadControllers.get(entry.key) === controller) loadControllers.delete(entry.key);
  }
}

async function expand(key: TreeKey): Promise<boolean> {
  const entry = index.value.entries.get(key);
  if (entry === undefined || isKeyDisabled(key) || !isExpandable(entry)) return false;
  const changed = expandedSet.value.has(key)
    ? false
    : setExpandedKeys([...expandedState.value.value, entry.key]);
  if (needsLoad(entry) && loadStates.get(key) !== "loading") await load(entry);
  return changed;
}

function collapse(key: TreeKey): boolean {
  const entry = index.value.entries.get(key);
  if (entry === undefined || !expandedSet.value.has(key) || isKeyDisabled(key)) return false;
  const active = registry.activeKey.value;
  if (active !== null && isTreeDescendant(index.value, key, active)) {
    if (focusWithin()) focusKey(key);
    else registry.setActiveKey(entry.key);
  }
  return setExpandedKeys(expandedState.value.value.filter((candidate) => candidate !== key));
}

async function toggle(key: TreeKey): Promise<boolean> {
  return expandedSet.value.has(key) ? collapse(key) : expand(key);
}

function expandAll(): boolean {
  const next = index.value.order.filter((key) => {
    const entry = index.value.entries.get(key);
    return entry !== undefined && entry.childKeys !== null && entry.childKeys.length > 0;
  });
  const merged = [...expandedState.value.value];
  for (const key of next) if (!expandedSet.value.has(key)) merged.push(key);
  return setExpandedKeys(merged);
}

function collapseAll(): boolean {
  const active = registry.activeKey.value;
  if (active !== null) {
    let topKey: TreeKey = active;
    let parent = index.value.entries.get(topKey)?.parentKey ?? null;
    while (parent !== null) {
      topKey = parent;
      parent = index.value.entries.get(parent)?.parentKey ?? null;
    }
    if (topKey !== active) {
      if (focusWithin()) focusKey(topKey);
      else {
        const entry = index.value.entries.get(topKey);
        if (entry !== undefined) registry.setActiveKey(entry.key);
      }
    }
  }
  return setExpandedKeys(noKeys);
}

function reload(key: TreeKey): Promise<boolean> {
  const entry = index.value.entries.get(key);
  if (entry === undefined || loadChildren === undefined) return Promise.resolve(false);
  loaded.delete(key);
  loadStates.delete(key);
  return load(entry);
}

function normalizeSelection(keys: readonly K[]): readonly K[] {
  if (selectionModeState.value === "none") return noKeys;
  const unique = [...new Set(keys)];
  return selectionModeState.value === "single" ? unique.slice(0, 1) : unique;
}

function setSelected(keys: readonly K[]): boolean {
  return selectedState.set(normalizeSelection(keys));
}

function selectOnly(key: TreeKey): boolean {
  const row = index.value.entries.get(key);
  if (row === undefined || isKeyDisabled(key) || selectionModeState.value === "none") return false;
  anchorKey.value = key;
  return setSelected([row.key]);
}

function toggleSelected(key: TreeKey): boolean {
  const row = index.value.entries.get(key);
  if (row === undefined || isKeyDisabled(key)) return false;
  if (selectionModeState.value !== "multiple") return selectOnly(key);
  anchorKey.value = key;
  const current = selectedState.value.value;
  return setSelected(
    selectedSet.value.has(key)
      ? current.filter((candidate) => candidate !== key)
      : [...current, row.key],
  );
}

function selectRange(key: TreeKey): boolean {
  if (selectionModeState.value !== "multiple") return selectOnly(key);
  const anchor =
    anchorKey.value !== null && rowByKey.value.has(anchorKey.value) ? anchorKey.value : key;
  const from = rowByKey.value.get(anchor)?.index ?? 0;
  const to = rowByKey.value.get(key)?.index ?? 0;
  const [start, end] = from <= to ? [from, to] : [to, from];
  const range = visibleItems.value
    .slice(start, end + 1)
    .filter((row) => !isKeyDisabled(row.key))
    .map((row) => row.key);
  return setSelected(range);
}

function selectAll(): boolean {
  if (selectionModeState.value !== "multiple") return false;
  const all = visibleItems.value.filter((row) => !isKeyDisabled(row.key)).map((row) => row.key);
  const everySelected = all.every((key) => selectedSet.value.has(key));
  return setSelected(everySelected ? noKeys : all);
}

function toggleChecked(key: TreeKey): boolean {
  if (!index.value.entries.has(key) || isKeyDisabled(key)) return false;
  return checkedState.set(
    toggleTreeChecked(index.value, checkedSet.value, key, {
      cascade: cascade.value,
      isDisabled: isKeyDisabled,
    }),
  );
}

function getCheckedState(key: TreeKey): TreeCheckedState {
  return checkedStates.value.get(key) ?? "unchecked";
}

function siblingsOf(key: TreeKey): readonly K[] {
  const parent = index.value.entries.get(key)?.parentKey ?? null;
  if (parent === null) return index.value.rootKeys;
  return index.value.entries.get(parent)?.childKeys ?? noKeys;
}

function moveByKeyboard(row: TreeFlatNode<T, K>, event: KeyboardEvent): boolean {
  if (reorder === undefined) return false;
  const siblings = siblingsOf(row.key);
  const position = siblings.indexOf(row.key);
  const previous = siblings[position - 1];
  const next = siblings[position + 1];
  const [forward, backward] =
    dirState.value === "rtl" ? ["ArrowLeft", "ArrowRight"] : ["ArrowRight", "ArrowLeft"];
  let targetKey: K | undefined;
  let placement: TreeDropPosition | undefined;
  if (event.key === "ArrowUp" && previous !== undefined) {
    targetKey = previous;
    placement = "before";
  } else if (event.key === "ArrowDown" && next !== undefined) {
    targetKey = next;
    placement = "after";
  } else if (event.key === forward && previous !== undefined) {
    targetKey = previous;
    placement = "inside";
  } else if (event.key === backward && row.parentKey !== null) {
    targetKey = row.parentKey;
    placement = "after";
  }
  if (targetKey === undefined || placement === undefined) return false;
  return reorder.move({
    key: row.key,
    targetKey,
    position: placement,
    source: "keyboard",
    originalEvent: event,
  });
}

function onFocusIn(event: FocusEvent): void {
  if (event.target === element.value) focus({ preventScroll: true });
}

function rowForElement(target: HTMLElement): TreeFlatNode<T, K> | undefined {
  for (const [key, candidate] of elements) {
    if (candidate === target) return rowByKey.value.get(key);
  }
  return undefined;
}

function onKeydown(event: KeyboardEvent): void {
  if (event.defaultPrevented || disabledState.value) return;
  const target = event.target;
  if (!(target instanceof HTMLElement) || target.getAttribute("role") !== "treeitem") return;
  const row = rowForElement(target);
  if (row === undefined) return;
  const rows = currentRows();
  const multiple = selectionModeState.value === "multiple";
  const [expandKey, collapseKey] =
    dirState.value === "rtl" ? ["ArrowLeft", "ArrowRight"] : ["ArrowRight", "ArrowLeft"];
  const moveTo = (next: TreeFlatNode<T, K> | undefined): void => {
    if (next === undefined) return;
    focusKey(next.key);
    if (event.shiftKey && multiple) toggleSelected(next.key);
    else if (selectionFollowsFocus && selectionModeState.value === "single") selectOnly(next.key);
  };

  if (event.altKey && event.key.startsWith("Arrow")) {
    if (moveByKeyboard(row, event)) event.preventDefault();
    return;
  }

  switch (event.key) {
    case "ArrowDown":
      moveTo(rows[row.index + 1]);
      break;
    case "ArrowUp":
      moveTo(rows[row.index - 1]);
      break;
    case expandKey:
      if (row.expandable && !row.expanded) void expand(row.key);
      else if (row.expanded) {
        const child = rows[row.index + 1];
        if (child !== undefined && child.parentKey === row.key) focusKey(child.key);
      }
      break;
    case collapseKey:
      if (row.expanded) collapse(row.key);
      else if (row.parentKey !== null) focusKey(row.parentKey);
      break;
    case "Home":
      if (rows[0] === undefined) break;
      focusKey(rows[0].key);
      if (multiple && event.shiftKey && (event.ctrlKey || event.metaKey)) selectRange(rows[0].key);
      break;
    case "End": {
      const last = rows.at(-1);
      if (last === undefined) break;
      focusKey(last.key);
      if (multiple && event.shiftKey && (event.ctrlKey || event.metaKey)) selectRange(last.key);
      break;
    }
    case "Enter":
      if (!isKeyDisabled(row.key)) {
        if (multiple) toggleSelected(row.key);
        else selectOnly(row.key);
        emit("action", row.key, row.node, event);
      }
      break;
    case " ":
      if (typeaheadController.query.value.length > 0) {
        typeaheadController.typeaheadProps.onKeydown(event);
        return;
      }
      if (checkableState.value) toggleChecked(row.key);
      else if (multiple && event.shiftKey) selectRange(row.key);
      else if (multiple) toggleSelected(row.key);
      else selectOnly(row.key);
      break;
    case "*":
      for (const sibling of siblingsOf(row.key)) void expand(sibling);
      break;
    default:
      if (multiple && (event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "a") {
        selectAll();
        break;
      }
      typeaheadController.typeaheadProps.onKeydown(event);
      return;
  }
  event.preventDefault();
}

function onItemClick(key: TreeKey, event: MouseEvent): void {
  if (isKeyDisabled(key)) return;
  focusKey(key, { preventScroll: true });
  if (selectionModeState.value === "multiple") {
    if (event.shiftKey) selectRange(key);
    else toggleSelected(key);
  } else if (selectionModeState.value === "single") {
    selectOnly(key);
  }
  if (expandOnClick) void toggle(key);
  if (event.detail === 2) {
    const entry = index.value.entries.get(key);
    if (entry !== undefined) emit("action", entry.key, entry.node, event);
  }
}

function registerElement(input: TreeItemElementInput): () => void {
  const stop = watch(
    input.element,
    (next, previous) => {
      if (previous !== null && elements.get(input.key) === previous) elements.delete(input.key);
      if (next !== null) elements.set(input.key, next);
    },
    { flush: "sync", immediate: true },
  );
  return () => {
    stop();
    if (elements.get(input.key) === input.element.value) elements.delete(input.key);
  };
}

function registerReorderItem(
  key: TreeKey,
  target: () => Element | null,
  label: () => string,
): TreeReorderItemRegistration | null {
  const entry = index.value.entries.get(key);
  if (reorder === undefined || entry === undefined) return null;
  return reorder.registerItem({
    key: entry.key,
    element: target,
    label,
    disabled: () => isKeyDisabled(key),
  });
}

function getDropPosition(key: TreeKey): TreeDropPosition | null {
  const target = reorder?.dropTarget.value ?? null;
  return target !== null && target.key === key ? target.position : null;
}

const treeProps = computed<{
  readonly role: "tree";
  readonly onFocus: (event: FocusEvent) => void;
  readonly onKeydown: (event: KeyboardEvent) => void;
}>(() => ({ role: "tree", onFocus: onFocusIn, onKeydown }));

onMounted(() => {
  virtualizer?.setViewport(element.value);
  // Lazy branches that start expanded load after hydration so server and client markup agree.
  watch(
    [expandedSet, index],
    () => {
      for (const key of expandedSet.value) {
        const entry = index.value.entries.get(key);
        if (entry !== undefined && needsLoad(entry) && !loadStates.has(key)) void load(entry);
      }
    },
    { immediate: true },
  );
});
onBeforeUnmount(() => virtualizer?.setViewport(null));
onScopeDispose(() => {
  for (const controller of loadControllers.values()) controller.abort();
  loadControllers.clear();
});

treeContext.provide({
  activeKey,
  checkable: checkableState,
  dir: dirState,
  disabled: disabledState,
  focusKey,
  getCheckedState,
  getDropPosition,
  getItemId,
  getLoadState: (key) => loadStates.get(key) ?? "idle",
  id: baseId,
  isDisabled: isKeyDisabled,
  isSelected: (key) => selectedSet.value.has(key),
  measureElement: (target, position) => virtualizer?.measureElement(target, position),
  onItemClick,
  onItemFocus: (key) => {
    const row = rowByKey.value.get(key);
    if (row !== undefined) registry.setActiveKey(row.key);
  },
  registerElement,
  registerReorderItem,
  reorderable: computed(() => reorder !== undefined),
  selectionMode: selectionModeState,
  toggleChecked: (key) => toggleChecked(key),
  toggleExpanded: (key) => {
    void toggle(key);
  },
  virtualized: computed(() => virtualizer !== undefined),
} satisfies TreeContextValue);

type TreeRootSetupExpose = Omit<
  TreeRootExpose<K>,
  "activeKey" | "checked" | "element" | "expanded" | "id" | "selected"
> & {
  readonly activeKey: ComputedRef<K | null>;
  readonly checked: ComputedRef<readonly K[]>;
  readonly element: typeof element;
  readonly expanded: ComputedRef<readonly K[]>;
  readonly id: ComputedRef<string>;
  readonly selected: ComputedRef<readonly K[]>;
};

const exposed = {
  activeKey,
  checked: checkedState.value,
  collapse,
  collapseAll,
  element,
  expand,
  expandAll,
  expanded: expandedState.value,
  focus,
  focusKey,
  getCheckedState,
  id: baseId,
  reload,
  selected: selectedState.value,
  setSelected,
  toggle,
  toggleChecked,
} satisfies TreeRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    v-bind="treeProps"
    :id="baseId"
    ref="element"
    :dir="dirState"
    :tabindex="rootTabindex"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-multiselectable="selectionModeState === 'multiple' ? 'true' : undefined"
    :aria-disabled="disabledState ? 'true' : undefined"
    data-vize-ui="tree"
    part="root"
    :data-state="state"
    :data-selection-mode="selectionModeState"
    :data-disabled="disabledState ? 'true' : undefined"
    :data-checkable="checkableState ? 'true' : undefined"
    :data-virtualized="virtualizer === undefined ? undefined : 'true'"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
