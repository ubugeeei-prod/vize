<script setup lang="ts" generic="T, Multiple extends boolean = false">
import { computed, onMounted, onScopeDispose, shallowRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { cascaderContext } from "./cascader-context.ts";
import type { CascaderContextValue } from "./cascader-context.ts";
import {
  areCascaderPathsEqual,
  createCascaderEquality,
  defaultCascaderChildren,
  defaultCascaderText,
  flattenCascaderPaths,
  fromCascaderSelection,
  indexOfCascaderPath,
  joinCascaderPath,
  normalizeCascaderText,
  searchCascaderPaths,
  serializeCascaderNode,
  toCascaderSelection,
} from "./cascader-model.ts";
import type { CascaderModelValue } from "./cascader-model.ts";
import type {
  CascaderColumnState,
  CascaderRootExpose,
  CascaderRootProps,
  CascaderSlotState,
  CascaderState,
} from "./cascader-types.ts";

const {
  id = undefined,
  options,
  modelValue = undefined,
  defaultValue = undefined,
  multiple = undefined,
  getChildren = undefined,
  isLeaf = undefined,
  loadChildren = undefined,
  by = undefined,
  itemText = undefined,
  itemDisabled = undefined,
  formValue = undefined,
  changeOnSelect = false,
  expandTrigger = "click",
  separator = " / ",
  search = undefined,
  open = undefined,
  defaultOpen = false,
  disabled = false,
  required = false,
  name = undefined,
  form = undefined,
  placeholder = undefined,
  typeaheadTimeout = 500,
} = defineProps<CascaderRootProps<T, Multiple>>();

const emit = defineEmits<{
  /** Fired when the selection requests a new controlled value (a path, or a list of paths). */
  "update:modelValue": [value: CascaderModelValue<T, Multiple>];

  /** Fired after a user interaction changes the selection: next value, previous value, native event. */
  change: [
    value: CascaderModelValue<T, Multiple>,
    previous: CascaderModelValue<T, Multiple>,
    nativeEvent: Event | null,
  ];

  /** Fired when the popup requests a controlled open value. */
  "update:open": [open: boolean];

  /** Fired when the search text requests a new controlled value. */
  "update:search": [query: string];

  /** Fired when `loadChildren` rejects for a branch, with the error. */
  "load-error": [node: T, error: unknown];
}>();

defineSlots<{
  /** Trigger, value, and popup parts. Receives columns, selection, and search results. */
  default?(props: CascaderSlotState<T>): unknown;
}>();

/** Highlighted option coordinates. */
interface ActivePosition {
  readonly level: number;
  readonly index: number;
}

/** In-flight lazy load for one level. */
interface PendingLoad {
  readonly node: T;
  readonly controller: AbortController;
}

/** One hidden form input. */
interface FormEntry {
  readonly key: string;
  readonly value: string;
}

const baseId = useDeterministicId({ id: () => id, hint: "cascader" });
const triggerId = computed(() => deriveDeterministicId(baseId.value, "trigger"));
const contentId = computed(() => deriveDeterministicId(baseId.value, "content"));
const multipleState = computed(() => multiple === true || String(multiple) === "");
const disabledState = computed(() => disabled);
const requiredState = computed(() => required);
const placeholderState = computed(() => placeholder);
const equals = computed(() => createCascaderEquality<T>(by));

const selection = useControllableState<readonly (readonly T[])[]>({
  value: () =>
    modelValue === undefined ? undefined : toCascaderSelection<T>(modelValue, multipleState.value),
  defaultValue: () => toCascaderSelection<T>(defaultValue, multipleState.value),
  equals: (left, right) =>
    left.length === right.length &&
    left.every((path, index) => areCascaderPathsEqual(path, right[index] ?? [], equals.value)),
  onChange: (next) =>
    emit("update:modelValue", fromCascaderSelection<T, Multiple>(next, multipleState.value)),
});
const selected = computed(() => selection.value.value);
const openState = useControllableState({
  value: () => open,
  defaultValue: () => defaultOpen,
});
const isOpen = computed(() => openState.value.value && !disabledState.value);
const state = computed<CascaderState>(() => (isOpen.value ? "open" : "closed"));
const searchState = useControllableState<string>({
  value: () => search,
  defaultValue: "",
  onChange: (next) => emit("update:search", next),
});

const expanded = shallowRef<readonly T[]>([]);
const active = shallowRef<ActivePosition | null>(null);
const childCache = shallowRef<ReadonlyMap<T, readonly T[]>>(new Map());
const loadingNodes = shallowRef<readonly T[]>([]);
const pendingLoads = new Map<number, PendingLoad>();
let pendingDescend: T | null = null;
let mounted = false;

const triggerElement = shallowRef<HTMLElement | null>(null);
const contentElement = shallowRef<HTMLElement | null>(null);

function currentActive(): ActivePosition | null {
  return active.value;
}

function currentLoading(): readonly T[] {
  return loadingNodes.value;
}

function currentLoader(): CascaderRootProps<T, Multiple>["loadChildren"] {
  return loadChildren;
}

function textOf(node: T): string {
  return itemText?.(node) ?? defaultCascaderText(node);
}

function isDisabled(node: T): boolean {
  return disabledState.value || itemDisabled?.(node) === true;
}

function staticChildren(node: T): readonly T[] | undefined {
  return getChildren === undefined ? defaultCascaderChildren(node) : getChildren(node);
}

function cachedChildren(node: T): readonly T[] | undefined {
  const exact = childCache.value.get(node);
  if (exact !== undefined) return exact;
  for (const [key, children] of childCache.value) {
    if (equals.value(key, node)) return children;
  }
  return undefined;
}

function childrenOf(node: T): readonly T[] | undefined {
  const children = staticChildren(node);
  if (children !== undefined && children.length > 0) return children;
  return cachedChildren(node);
}

function isLoading(node: T): boolean {
  return loadingNodes.value.some((candidate) => equals.value(candidate, node));
}

function isBranch(node: T): boolean {
  if (isLeaf !== undefined) return !isLeaf(node);
  const children = staticChildren(node);
  if (children !== undefined && children.length > 0) return true;
  if (loadChildren === undefined) return false;
  const cached = cachedChildren(node);
  return cached === undefined || cached.length > 0;
}

const columns = computed<readonly CascaderColumnState<T>[]>(() => {
  const list: CascaderColumnState<T>[] = [{ level: 0, loading: false, options, parent: null }];
  for (const [index, node] of expanded.value.entries()) {
    if (!isBranch(node)) break;
    list.push({
      level: index + 1,
      loading: isLoading(node),
      options: childrenOf(node) ?? [],
      parent: node,
    });
  }
  return Object.freeze(list);
});

function optionsAt(level: number): readonly T[] {
  return columns.value[level]?.options ?? [];
}

function indexIn(level: number, node: T): number {
  return optionsAt(level).findIndex((candidate) => equals.value(candidate, node));
}

function optionId(level: number, node: T): string {
  const index = indexIn(level, node);
  return deriveDeterministicId(baseId.value, `option-${level}-${index < 0 ? "x" : index}`);
}

function columnId(level: number): string {
  return deriveDeterministicId(baseId.value, `column-${level}`);
}

function activeNode(): T | undefined {
  const position = currentActive();
  return position === null ? undefined : optionsAt(position.level)[position.index];
}

const activeDescendant = computed(() => {
  const position = currentActive();
  if (!isOpen.value || position === null) return undefined;
  const node = optionsAt(position.level)[position.index];
  return node === undefined ? undefined : optionId(position.level, node);
});
const columnIds = computed(() => columns.value.map((column) => columnId(column.level)));
const selectedText = computed(() =>
  selected.value.map((path) => joinCascaderPath(path, textOf, separator)),
);
const searchResults = computed(() =>
  searchCascaderPaths(
    flattenCascaderPaths(options, childrenOf, changeOnSelect && !multipleState.value),
    searchState.value.value,
    textOf,
  ),
);
const formEntries = computed<readonly FormEntry[]>(() =>
  selected.value.map((path, index) => ({
    key: String(index),
    value: path.map((node) => formValue?.(node) ?? serializeCascaderNode(node, by)).join(separator),
  })),
);

function currentSelection(): readonly (readonly T[])[] {
  return selection.value.value;
}

function currentOpen(): boolean {
  return isOpen.value;
}

function setOpen(value: boolean, nativeEvent: Event | null = null): boolean {
  void nativeEvent;
  const previous = currentOpen();
  const next = value && !disabledState.value;
  const changed = openState.set(next);
  if (changed || previous !== next) emit("update:open", next);
  return changed || previous !== next;
}

function commit(next: readonly (readonly T[])[], nativeEvent: Event | null): boolean {
  const previous = currentSelection();
  const changed = selection.set(Object.freeze(next));
  if (changed) {
    emit(
      "change",
      fromCascaderSelection<T, Multiple>(next, multipleState.value),
      fromCascaderSelection<T, Multiple>(previous, multipleState.value),
      nativeEvent,
    );
  }
  return changed;
}

function selectPath(path: readonly T[], nativeEvent: Event | null = null): boolean {
  if (disabledState.value || path.length === 0 || path.some(isDisabled)) return false;
  const frozen = Object.freeze([...path]);
  if (!multipleState.value) {
    const changed = commit([frozen], nativeEvent);
    if (!isBranch(frozen[frozen.length - 1] as T)) setOpen(false, nativeEvent);
    return changed;
  }
  const current = currentSelection();
  const index = indexOfCascaderPath(current, frozen, equals.value);
  return commit(
    index < 0 ? [...current, frozen] : current.filter((_, candidate) => candidate !== index),
    nativeEvent,
  );
}

function abortLoad(level: number): void {
  const pending = pendingLoads.get(level);
  if (pending === undefined) return;
  pending.controller.abort();
  pendingLoads.delete(level);
  loadingNodes.value = loadingNodes.value.filter(
    (candidate) => !equals.value(candidate, pending.node),
  );
}

function load(level: number, node: T): void {
  const loader = currentLoader();
  if (loader === undefined || !mounted || isLoading(node)) return;
  if (cachedChildren(node) !== undefined) return;
  const staticList = staticChildren(node);
  if (staticList !== undefined && staticList.length > 0) return;
  abortLoad(level);
  const controller = new AbortController();
  pendingLoads.set(level, { controller, node });
  loadingNodes.value = currentLoading().concat([node]);
  const finish = () => {
    if (pendingLoads.get(level)?.controller === controller) pendingLoads.delete(level);
    loadingNodes.value = loadingNodes.value.filter((candidate) => !equals.value(candidate, node));
  };
  void Promise.resolve()
    .then(() => loader(node, { signal: controller.signal }))
    .then(
      (children) => {
        if (controller.signal.aborted) return;
        const next = new Map(childCache.value);
        next.set(node, Object.freeze([...children]));
        childCache.value = next;
        finish();
        if (pendingDescend !== null && equals.value(pendingDescend, node)) {
          pendingDescend = null;
          const current = activeNode();
          if (current !== undefined && equals.value(current, node)) descend(level);
        }
      },
      (error: unknown) => {
        if (controller.signal.aborted) return;
        finish();
        emit("load-error", node, error);
      },
    );
}

function expand(level: number, node: T): void {
  for (const pendingLevel of Array.from(pendingLoads.keys())) {
    const pending = pendingLoads.get(pendingLevel);
    if (
      pendingLevel > level ||
      (pendingLevel === level && pending !== undefined && !equals.value(pending.node, node))
    ) {
      abortLoad(pendingLevel);
    }
  }
  expanded.value = Object.freeze([...expanded.value.slice(0, level), node]);
  if (isBranch(node)) load(level, node);
}

function collapseTo(level: number): void {
  if (expanded.value.length > level) expanded.value = Object.freeze(expanded.value.slice(0, level));
  for (const pendingLevel of Array.from(pendingLoads.keys())) {
    if (pendingLevel >= level) abortLoad(pendingLevel);
  }
}

function firstEnabled(level: number, from = 0, step: 1 | -1 = 1): number | null {
  const list = optionsAt(level);
  for (let index = from; index >= 0 && index < list.length; index += step) {
    const node = list[index];
    if (node !== undefined && !isDisabled(node)) return index;
  }
  return null;
}

function setActive(level: number, index: number | null): void {
  active.value = index === null ? null : { index, level };
  if (index !== null)
    collapseTo(level + (expanded.value.length > level && sameAsExpanded(level, index) ? 1 : 0));
}

function sameAsExpanded(level: number, index: number): boolean {
  const node = optionsAt(level)[index];
  const opened = expanded.value[level];
  return node !== undefined && opened !== undefined && equals.value(node, opened);
}

function descend(level: number): void {
  const index = firstEnabled(level + 1);
  if (index !== null) active.value = { index, level: level + 1 };
}

function pathTo(level: number, node: T): readonly T[] {
  return Object.freeze([...expanded.value.slice(0, level), node]);
}

function initializeOnOpen(): void {
  const path = currentSelection()[0];
  expanded.value = Object.freeze([]);
  active.value = null;
  if (path !== undefined && path.length > 0) {
    for (const [level, node] of path.slice(0, -1).entries()) expand(level, node);
    const level = path.length - 1;
    const last = path[level] as T;
    const index = indexIn(level, last);
    if (index >= 0) {
      active.value = { index, level };
      return;
    }
  }
  const index = firstEnabled(0);
  active.value = index === null ? null : { index, level: 0 };
}

watch(
  isOpen,
  (next) => {
    if (next) {
      initializeOnOpen();
      return;
    }
    active.value = null;
    pendingDescend = null;
  },
  { flush: "sync", immediate: true },
);

let typeaheadBuffer = "";
let typeaheadTimer: ReturnType<typeof setTimeout> | null = null;

function typeahead(key: string): boolean {
  const position = currentActive();
  const level = position?.level ?? 0;
  const list = optionsAt(level);
  if (list.length === 0) return false;
  const repeated = typeaheadBuffer.length > 0 && [...typeaheadBuffer].every((char) => char === key);
  typeaheadBuffer = repeated ? key : `${typeaheadBuffer}${key}`;
  if (typeaheadTimer !== null) clearTimeout(typeaheadTimer);
  typeaheadTimer = setTimeout(() => {
    typeaheadBuffer = "";
    typeaheadTimer = null;
  }, typeaheadTimeout);
  const needle = normalizeCascaderText(typeaheadBuffer);
  const start = (position?.index ?? -1) + (typeaheadBuffer.length === 1 ? 1 : 0);
  for (let offset = 0; offset < list.length; offset++) {
    const index = (start + offset + list.length) % list.length;
    const node = list[index];
    if (node === undefined || isDisabled(node)) continue;
    if (normalizeCascaderText(textOf(node)).startsWith(needle)) {
      setActive(level, index);
      return true;
    }
  }
  return true;
}

function resetTypeahead(): void {
  typeaheadBuffer = "";
  if (typeaheadTimer !== null) clearTimeout(typeaheadTimer);
  typeaheadTimer = null;
}

onScopeDispose(resetTypeahead);

function isPrintable(event: KeyboardEvent): boolean {
  return (
    event.key.length === 1 && event.key !== " " && !event.ctrlKey && !event.metaKey && !event.altKey
  );
}

function activateNode(level: number, node: T, event: Event): void {
  if (isBranch(node)) {
    expand(level, node);
    if (childrenOf(node) === undefined) pendingDescend = node;
    else descend(level);
    if (changeOnSelect && !multipleState.value) selectPath(pathTo(level, node), event);
    return;
  }
  selectPath(pathTo(level, node), event);
}

function onTriggerKeydown(event: KeyboardEvent): void {
  if (disabledState.value || event.defaultPrevented || event.isComposing) return;
  const { key } = event;
  if (!isOpen.value) {
    if (key === "ArrowDown" || key === "ArrowUp" || key === "Enter" || key === " ") {
      event.preventDefault();
      setOpen(true, event);
    }
    return;
  }
  // Navigation keys end a pending typeahead query, so each column starts fresh.
  if (!isPrintable(event) && key !== " ") resetTypeahead();
  const position = currentActive() ?? { index: -1, level: 0 };
  const node = activeNode();
  switch (key) {
    case "ArrowDown":
    case "ArrowUp": {
      event.preventDefault();
      const step = key === "ArrowDown" ? 1 : -1;
      const index = firstEnabled(position.level, position.index + step, step);
      if (index !== null) setActive(position.level, index);
      return;
    }
    case "Home":
    case "End": {
      event.preventDefault();
      const list = optionsAt(position.level);
      const index =
        key === "Home"
          ? firstEnabled(position.level)
          : firstEnabled(position.level, list.length - 1, -1);
      if (index !== null) setActive(position.level, index);
      return;
    }
    case "ArrowRight":
      event.preventDefault();
      if (node !== undefined && isBranch(node) && !isDisabled(node)) {
        expand(position.level, node);
        if (childrenOf(node) === undefined) pendingDescend = node;
        else descend(position.level);
      }
      return;
    case "ArrowLeft":
      event.preventDefault();
      if (position.level > 0) {
        const parent = expanded.value[position.level - 1];
        const index = parent === undefined ? -1 : indexIn(position.level - 1, parent);
        pendingDescend = null;
        if (index >= 0) {
          active.value = { index, level: position.level - 1 };
          collapseTo(position.level - 1);
        }
      }
      return;
    case "Enter":
    case " ":
      if (key === " " && typeaheadBuffer.length > 0) {
        event.preventDefault();
        typeahead(key);
        return;
      }
      event.preventDefault();
      if (node !== undefined && !isDisabled(node)) activateNode(position.level, node, event);
      return;
    case "Escape":
      event.preventDefault();
      setOpen(false, event);
      return;
    case "Tab":
      setOpen(false, event);
      return;
    default:
      if (isPrintable(event)) {
        event.preventDefault();
        typeahead(key);
      }
  }
}

function pressItem(level: number, node: T, event: Event): void {
  if (isDisabled(node)) return;
  const index = indexIn(level, node);
  if (index >= 0) active.value = { index, level };
  if (isBranch(node)) {
    expand(level, node);
    if (changeOnSelect && !multipleState.value) selectPath(pathTo(level, node), event);
    return;
  }
  selectPath(pathTo(level, node), event);
}

function hoverItem(level: number, node: T): void {
  if (isDisabled(node)) return;
  const index = indexIn(level, node);
  if (index >= 0) active.value = { index, level };
  if (expandTrigger === "hover" && isBranch(node)) expand(level, node);
}

function sameExpanded(level: number, node: T): boolean {
  const opened = expanded.value[level];
  return opened !== undefined && equals.value(opened, node);
}

function prefixMatches(path: readonly T[], level: number, node: T): boolean {
  return areCascaderPathsEqual(path.slice(0, level + 1), pathTo(level, node), equals.value);
}

onMounted(() => {
  mounted = true;
  if (isOpen.value) {
    for (const [level, node] of expanded.value.entries()) {
      if (isBranch(node)) load(level, node);
    }
  }
});

onScopeDispose(() => {
  mounted = false;
  for (const level of Array.from(pendingLoads.keys())) abortLoad(level);
});

const context: CascaderContextValue<T> = {
  activeDescendant,
  baseId,
  columnId,
  columnIds,
  columns,
  contentElement,
  contentId,
  disabled: disabledState,
  hoverItem,
  isActive: (level, node) => {
    const position = currentActive();
    const current = position === null || position.level !== level ? undefined : activeNode();
    return current !== undefined && equals.value(current, node);
  },
  isBranch,
  isDisabled,
  isExpanded: sameExpanded,
  isLoading,
  isPartial: (level, node) =>
    selected.value.some((path) => path.length > level + 1 && prefixMatches(path, level, node)),
  isSelectedEnd: (level, node) =>
    selected.value.some((path) => path.length === level + 1 && prefixMatches(path, level, node)),
  multiple: multipleState,
  onTriggerKeydown,
  open: isOpen,
  optionId,
  placeholder: placeholderState,
  pressItem,
  required: requiredState,
  selectedText,
  setOpen,
  state,
  textOf,
  triggerElement,
  triggerId,
};
cascaderContext.provide(context);

const slotState = computed<CascaderSlotState<T>>(() => ({
  columns: columns.value,
  disabled: disabledState.value,
  open: isOpen.value,
  searchResults: searchResults.value,
  selected: selected.value,
  selectedText: selectedText.value,
  selectPath: (path: readonly T[]) => selectPath(path, null),
  state: state.value,
}));

type CascaderRootSetupExpose = Omit<CascaderRootExpose<T>, "open" | "selected"> & {
  readonly open: ComputedRef<boolean>;
  readonly selected: ComputedRef<readonly (readonly T[])[]>;
};

const exposed = {
  clear: () => commit([], null),
  expandPath: (path: readonly T[]) => {
    for (const [level, node] of path.entries()) expand(level, node);
  },
  focus: (focusOptions?: FocusOptions) => triggerElement.value?.focus(focusOptions),
  open: isOpen,
  reset: () => selection.reset(),
  selected,
  selectPath: (path: readonly T[]) => selectPath(path, null),
  setOpen: (value: boolean) => setOpen(value, null),
} satisfies CascaderRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="cascader"
    part="root"
    :data-state="state"
    :data-selection-mode="multipleState ? 'multiple' : 'single'"
    :data-disabled="disabledState ? 'true' : undefined"
    :data-empty="selected.length === 0 ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
    <template v-if="name !== undefined">
      <input
        v-for="entry in formEntries as readonly FormEntry[]"
        :key="entry.key"
        type="hidden"
        data-vize-ui="cascader-native"
        part="native"
        :name
        :form
        :value="entry.value"
        :disabled="disabledState"
      />
    </template>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
