<script setup lang="ts" generic="T, Multiple extends boolean = false">
import { computed, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { useTypeahead } from "../../interaction/typeahead/typeahead.ts";
import { listboxGridContext } from "./listbox-grid-context.ts";
import type { ListboxGridContextValue } from "./listbox-grid-context.ts";
import {
  createGridEquality,
  fromGridSelection,
  gridMoveFromKey,
  moveInGrid,
  serializeGridValue,
  toGridSelection,
} from "./listbox-grid-model.ts";
import type { ListboxGridModelValue } from "./listbox-grid-model.ts";
import type {
  ListboxGridExpose,
  ListboxGridProps,
  ListboxGridSlotState,
  ListboxGridState,
} from "./listbox-grid-types.ts";

const {
  id = undefined,
  modelValue = undefined,
  defaultValue = undefined,
  multiple = undefined,
  by = undefined,
  columns = 4,
  pageRows = 3,
  selectionFollowsFocus = false,
  disabled = false,
  required = false,
  name = undefined,
  form = undefined,
  formValue = undefined,
  dir = "ltr",
  typeaheadTimeout = 500,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<ListboxGridProps<T, Multiple>>();

const emit = defineEmits<{
  /** Fired when the selection requests a new controlled value (`T | null` or `readonly T[]`). */
  "update:modelValue": [value: ListboxGridModelValue<T, Multiple>];

  /** Fired after a user interaction changes the selection: next value, previous value, native event. */
  change: [
    value: ListboxGridModelValue<T, Multiple>,
    previous: ListboxGridModelValue<T, Multiple>,
    nativeEvent: Event,
  ];
}>();

defineSlots<{
  /** `ListboxGridItem` options. Receives selection, active option, and layout state. */
  default?(props: ListboxGridSlotState<T>): unknown;
}>();

/** Registered option data. */
interface GridEntry {
  readonly id: ComputedRef<string>;
  readonly value: T;
}

/** One hidden form input. */
interface FormEntry {
  readonly key: string;
  readonly value: string;
}

const element = useTemplateRef<HTMLDivElement>("element");
const gridId = useDeterministicId({ id: () => id, hint: "listbox-grid" });
const multipleState = computed(() => multiple === true || String(multiple) === "");
const disabledState = computed(() => disabled);
const columnsState = computed(() =>
  Number.isFinite(columns) && columns >= 1 ? Math.floor(columns) : 1,
);
const equals = computed(() => createGridEquality<T>(by));
const selection = useControllableState<readonly T[]>({
  value: () =>
    modelValue === undefined ? undefined : toGridSelection<T>(modelValue, multipleState.value),
  defaultValue: () => toGridSelection<T>(defaultValue, multipleState.value),
  equals: (left, right) =>
    left.length === right.length &&
    left.every((value, index) => equals.value(value, right[index] as T)),
  onChange: (next) =>
    emit("update:modelValue", fromGridSelection<T, Multiple>(next, multipleState.value)),
});
const selected = computed(() => selection.value.value);
const registry = createCollectionRegistry<string, GridEntry>();
const typeahead = useTypeahead({
  registry,
  isDisabled: disabledState,
  timeout: () => typeaheadTimeout,
});
const activeId = computed(() => registry.activeKey.value);
const state = computed<ListboxGridState>(() => {
  if (disabledState.value) return "disabled";
  return selected.value.length === 0 ? "empty" : "selected";
});
const formEntries = computed<readonly FormEntry[]>(() =>
  selected.value.map((value, index) => ({
    key: String(index),
    value: formValue?.(value) ?? serializeGridValue(value, by),
  })),
);
const slotState = computed<ListboxGridSlotState<T>>(() => ({
  activeId: activeId.value,
  columns: columnsState.value,
  disabled: disabledState.value,
  selected: selected.value,
  state: state.value,
}));
const interactiveProps = computed(() => ({
  role: "listbox" as const,
  ...(disabledState.value ? {} : { tabindex: 0 as const }),
  onFocus,
  onKeydown,
}));

function includes(value: T): boolean {
  return selected.value.some((candidate) => equals.value(candidate, value));
}

function currentSelection(): readonly T[] {
  return selection.value.value;
}

function commit(next: readonly T[], nativeEvent: Event | null): boolean {
  const previous = currentSelection();
  const changed = selection.set(Object.freeze([...next]));
  if (changed && nativeEvent !== null) {
    emit(
      "change",
      fromGridSelection<T, Multiple>(next, multipleState.value),
      fromGridSelection<T, Multiple>(previous, multipleState.value),
      nativeEvent,
    );
  }
  return changed;
}

function choose(value: T, nativeEvent: Event | null): boolean {
  if (disabledState.value) return false;
  if (!multipleState.value) return commit([value], nativeEvent);
  const current = currentSelection();
  return commit(
    includes(value)
      ? current.filter((candidate) => !equals.value(candidate, value))
      : [...current, value],
    nativeEvent,
  );
}

function addToSelection(value: T, nativeEvent: Event): void {
  if (!includes(value)) commit([...currentSelection(), value], nativeEvent);
}

function activeIndex(): number {
  const key = registry.activeKey.value;
  return key === null ? -1 : registry.items.value.findIndex((item) => item.key === key);
}

function activate(key: string): void {
  if (disabledState.value) return;
  if (registry.navigableItems.value.some((item) => item.key === key)) registry.setActiveKey(key);
}

function activeEntry(): GridEntry | null {
  const key = registry.activeKey.value;
  return key === null ? null : (registry.getItem(key)?.value ?? null);
}

function reveal(): void {
  const key = registry.activeKey.value;
  const target = key === null ? null : registry.getItem(key)?.element;
  const scroll = (target as Partial<HTMLElement> | null | undefined)?.scrollIntoView;
  if (target && typeof scroll === "function") scroll.call(target, { block: "nearest" });
}

function onFocus(event: FocusEvent): void {
  if (event.target !== event.currentTarget || registry.activeKey.value !== null) return;
  const firstSelected = registry.navigableItems.value.find((item) => includes(item.value.value));
  const target = firstSelected ?? registry.navigableItems.value[0];
  if (target !== undefined) registry.setActiveKey(target.key);
}

function onKeydown(event: KeyboardEvent): void {
  if (disabledState.value || event.defaultPrevented || event.target !== event.currentTarget) {
    return;
  }
  const move = gridMoveFromKey(event);
  if (move !== null) {
    const itemsList = registry.items.value;
    const target = moveInGrid(move, {
      columns: columnsState.value,
      count: itemsList.length,
      direction: dir,
      index: activeIndex(),
      isEnabled: (index) => itemsList[index]?.disabled === false,
      pageRows,
    });
    event.preventDefault();
    const next = target === null ? undefined : itemsList[target];
    if (next === undefined) return;
    registry.setActiveKey(next.key);
    reveal();
    if (event.shiftKey && multipleState.value) addToSelection(next.value.value, event);
    else if (selectionFollowsFocus && !multipleState.value) choose(next.value.value, event);
    return;
  }
  if (event.key === "Enter" || (event.key === " " && typeahead.query.value.length === 0)) {
    const entry = activeEntry();
    if (entry === null) return;
    event.preventDefault();
    choose(entry.value, event);
    return;
  }
  if (multipleState.value && (event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "a") {
    event.preventDefault();
    commit(
      registry.navigableItems.value.map((item) => item.value.value),
      event,
    );
    return;
  }
  typeahead.typeaheadProps.onKeydown(event);
  if (event.defaultPrevented) reveal();
}

watch(disabledState, (next) => {
  if (next) registry.setActiveKey(null);
});

const context: ListboxGridContextValue<T> = {
  activate,
  activeId,
  choose,
  columns: columnsState,
  disabled: disabledState,
  focus: () => element.value?.focus({ preventScroll: true }),
  indexOf: (key) => registry.items.value.findIndex((item) => item.key === key),
  isSelected: includes,
  multiple: multipleState,
  register: ({ disabled: itemDisabled, element: itemElement, id: itemId, textValue, value }) =>
    registry.register({
      disabled: itemDisabled,
      element: itemElement,
      key: itemId.value,
      textValue,
      value: { id: itemId, value },
    }),
};
listboxGridContext.provide(context);

type ListboxGridSetupExpose = Omit<ListboxGridExpose<T>, "activeId" | "selected"> & {
  readonly activeId: ComputedRef<string | null>;
  readonly selected: ComputedRef<readonly T[]>;
};

const exposed = {
  activeId,
  clear: () => commit([], null),
  focus: (options?: FocusOptions) => element.value?.focus(options),
  reset: () => selection.reset(),
  select: (value: T) => choose(value, null),
  selected,
} satisfies ListboxGridSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="gridId"
    ref="element"
    v-bind="interactiveProps"
    :dir
    :aria-activedescendant="disabledState ? undefined : (activeId ?? undefined)"
    :aria-multiselectable="multipleState ? 'true' : undefined"
    :aria-disabled="disabledState ? 'true' : undefined"
    :aria-required="required ? 'true' : undefined"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    data-vize-ui="listbox-grid"
    part="root"
    :data-state="state"
    :data-columns="columnsState"
    :data-selection-mode="multipleState ? 'multiple' : 'single'"
    :data-disabled="disabledState ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
    <template v-if="name !== undefined">
      <input
        v-for="entry in formEntries as readonly FormEntry[]"
        :key="entry.key"
        type="hidden"
        data-vize-ui="listbox-grid-native"
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
