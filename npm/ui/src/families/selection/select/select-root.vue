<script setup lang="ts" generic="T, Multiple extends boolean = false">
import { computed, onMounted, onUnmounted, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { useSelectCollection } from "./select-collection.ts";
import { useSelectSelection } from "./select-selection.ts";
import type { SelectOpenFocus } from "./select-collection.ts";
import { selectContext } from "./select-context.ts";
import type { SelectContextValue } from "./select-context.ts";
import {
  createSelectValueEquality,
  defaultSelectText,
  fromSelectList,
  includesSelectValue,
  indexOfSelectValue,
  removeFromList,
  serializeSelectValue,
  toggleInList,
  toSelectList,
} from "./select-model.ts";
import type { SelectModelValue } from "./select-model.ts";
import { isPrintableKey, readBooleanProp } from "./select-keyboard.ts";
import type {
  SelectDirection,
  SelectRootExpose,
  SelectRootProps,
  SelectSelectionMode,
  SelectSlotState,
  SelectState,
} from "./select-types.ts";

const {
  id = undefined,
  modelValue = undefined,
  defaultValue = undefined,
  multiple = undefined,
  items = undefined,
  by = undefined,
  itemText = undefined,
  itemDisabled = undefined,
  formValue = undefined,
  open = undefined,
  defaultOpen = false,
  disabled = false,
  required = false,
  name = undefined,
  form = undefined,
  autocomplete = undefined,
  placeholder = undefined,
  loop = false,
  typeaheadTimeout = 500,
  selectOnTab = false,
  closeOnSelect = undefined,
  dir = "ltr",
  ariaInvalid = false,
} = defineProps<SelectRootProps<T, Multiple>>();

const emit = defineEmits<{
  /** Fired when the selection requests a new controlled value (`T | null` or `readonly T[]`). */
  "update:modelValue": [value: SelectModelValue<T, Multiple>];

  /** Fired after a user interaction changes the selection: next value, previous value, and the native event. */
  change: [
    value: SelectModelValue<T, Multiple>,
    previous: SelectModelValue<T, Multiple>,
    nativeEvent: Event,
  ];

  /** Fired when the popup requests a controlled open value. */
  "update:open": [open: boolean];

  /** Fired after any distinct open-state request with the next state, previous state, and triggering event. */
  "open-change": [open: boolean, previous: boolean, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Compound Select parts. Receives the current selection and open state. */
  default?(props: SelectSlotState<T>): unknown;
}>();

const nativeElement = useTemplateRef<HTMLSelectElement>("nativeElement");
const baseId = useDeterministicId({ id: () => id, hint: "select" });
const triggerId = computed(() => deriveDeterministicId(baseId.value, "trigger"));
const listboxId = computed(() => deriveDeterministicId(baseId.value, "listbox"));
const multipleState = computed(() => readBooleanProp(multiple));
const disabledState = computed(() => disabled);
const requiredState = computed(() => required);
const directionState = computed<SelectDirection>(() => dir);
const placeholderState = computed(() => placeholder);
const itemsState = computed(() => items);
const equals = computed(() => createSelectValueEquality<T>(by));
const userInvalid = shallowRef(false);

const selection = useSelectSelection<T, Multiple>({
  defaultValue: () => defaultValue,
  equals: () => equals.value,
  modelValue: () => modelValue,
  multiple: () => multipleState.value,
  onChange: (next) => emit("update:modelValue", next),
});
const selected = computed(() => selection.value.value);

const openState = useControllableState({
  value: () => open,
  defaultValue: () => defaultOpen,
});
const isOpen = computed(() => openState.value.value && !disabledState.value);
const state = computed<SelectState>(() => (isOpen.value ? "open" : "closed"));
const invalid = computed(() => ariaInvalid !== false || userInvalid.value);
const selectionMode = computed<SelectSelectionMode>(() =>
  multipleState.value ? "multiple" : "single",
);

const collection = useSelectCollection<T>({
  disabled: disabledState,
  loop: () => loop,
  typeahead: () => true,
  typeaheadTimeout: () => typeaheadTimeout,
});

const textCache = shallowRef<ReadonlyMap<T, string>>(new Map());

function serialize(value: T): string {
  return formValue?.(value) ?? serializeSelectValue(value, by);
}

function registeredText(value: T): string | undefined {
  for (const item of collection.registry.items.value) {
    if (equals.value(item.value.value, value)) {
      return item.textValue.length > 0 ? item.textValue : undefined;
    }
  }
  return undefined;
}

function textFor(value: T): string {
  return (
    itemText?.(value) ?? registeredText(value) ?? cachedText(value) ?? defaultSelectText(value, by)
  );
}

function cachedText(value: T): string | undefined {
  for (const [candidate, text] of textCache.value) {
    if (equals.value(candidate, value)) return text;
  }
  return undefined;
}

const selectedText = computed(() => selected.value.map(textFor));
const activeDescendant = computed(() => {
  if (!isOpen.value) return undefined;
  return collection.activeItem.value?.value.id.value;
});
// A named select mirrors every `items` value so browser autofill can pick
// one; otherwise only the selection is mirrored, keeping large lists cheap.
const nativeOptions = computed<readonly NativeOption[]>(() => {
  const source = name !== undefined && items !== undefined ? items : selected.value;
  return source.map((value, index) => ({
    key: name === undefined ? String(index) : serialize(value),
    selected: includesSelectValue(selected.value, value, equals.value),
  }));
});
/** One mirrored option in the hidden native select. */
interface NativeOption {
  readonly key: string;
  readonly selected: boolean;
}

const nativeLabel = computed(() => placeholder ?? name ?? baseId.value);
const slotState = computed<SelectSlotState<T>>(() => ({
  disabled: disabledState.value,
  invalid: invalid.value,
  open: isOpen.value,
  required: requiredState.value,
  selected: selected.value,
  selectedText: selectedText.value,
  selectionMode: selectionMode.value,
  state: state.value,
}));

const triggerElement = shallowRef<HTMLElement | null>(null);
const contentElement = shallowRef<HTMLElement | null>(null);
const viewportElement = shallowRef<HTMLElement | null>(null);

function currentSelection(): readonly T[] {
  return selection.value.value;
}

function currentOpen(): boolean {
  return isOpen.value;
}

function nativeSelect(): HTMLSelectElement | null {
  return nativeElement.value;
}

function commit(next: readonly T[], nativeEvent: Event | null): boolean {
  const previous = currentSelection();
  const changed = selection.set(next);
  if (changed) {
    userInvalid.value = false;
    if (nativeEvent !== null) {
      emit(
        "change",
        fromSelectList<T, Multiple>(next, multipleState.value),
        fromSelectList<T, Multiple>(previous, multipleState.value),
        nativeEvent,
      );
    }
  }
  return changed;
}

function setOpen(value: boolean, nativeEvent: Event | null = null): boolean {
  const previous = currentOpen();
  const next = value && !disabledState.value;
  const changed = openState.set(next);
  if (changed || previous !== next) {
    emit("update:open", next);
    emit("open-change", next, previous, nativeEvent);
  }
  return changed || previous !== next;
}

function selectedIndex(): number | null {
  const first = selected.value[0];
  if (items === undefined || first === undefined) return null;
  const index = indexOfSelectValue(items, first, equals.value);
  return index < 0 ? null : index;
}

let nextOpenIntent: SelectOpenFocus = "selected";
let nextOpenGrapheme: string | null = null;

function openWith(
  intent: SelectOpenFocus,
  nativeEvent: Event | null,
  grapheme: string | null = null,
): void {
  nextOpenIntent = intent;
  nextOpenGrapheme = grapheme;
  setOpen(true, nativeEvent);
  nextOpenIntent = "selected";
  nextOpenGrapheme = null;
}

function choose(value: T, nativeEvent: Event | null): boolean {
  if (disabledState.value) return false;
  const changed = commit(
    toggleInList(selected.value, value, multipleState.value, equals.value),
    nativeEvent,
  );
  if (closeOnSelect ?? !multipleState.value) setOpen(false, nativeEvent);
  return changed;
}

function chooseActive(nativeEvent: Event): boolean {
  const item = collection.activeItem.value;
  if (item === null || item.disabled) return false;
  choose(item.value.value, nativeEvent);
  return true;
}

function onTriggerKeydown(event: KeyboardEvent): void {
  if (disabledState.value || event.defaultPrevented || event.isComposing) return;
  const { key } = event;
  if (!isOpen.value) {
    if (key === "ArrowDown" || key === "ArrowUp" || key === "Enter" || key === " ") {
      event.preventDefault();
      openWith("selected", event);
      return;
    }
    if (key === "Home" || key === "End") {
      event.preventDefault();
      openWith(key === "Home" ? "first" : "last", event);
      return;
    }
    if (isPrintableKey(event)) {
      event.preventDefault();
      openWith("selected", event, key);
    }
    return;
  }
  switch (key) {
    case "ArrowDown":
    case "ArrowUp":
      event.preventDefault();
      if (event.altKey) {
        if (key === "ArrowUp" && !multipleState.value) chooseActive(event);
        if (key === "ArrowUp") setOpen(false, event);
        return;
      }
      collection.navigate(key === "ArrowDown" ? "next" : "previous", event);
      return;
    case "Home":
    case "End":
      event.preventDefault();
      collection.navigate(key === "Home" ? "first" : "last", event);
      return;
    case "PageDown":
    case "PageUp":
      event.preventDefault();
      collection.navigate(key === "PageDown" ? "page-next" : "page-previous", event);
      return;
    case "Enter":
      event.preventDefault();
      chooseActive(event);
      return;
    case " ":
      if (collection.typeaheadQuery.value.length > 0) {
        collection.typeahead(event);
        return;
      }
      event.preventDefault();
      chooseActive(event);
      return;
    case "Escape":
      event.preventDefault();
      setOpen(false, event);
      return;
    case "Tab":
      if (selectOnTab && !multipleState.value) chooseActive(event);
      setOpen(false, event);
      return;
    default:
      collection.typeahead(event);
  }
}

function rememberText(value: T, text: string): void {
  if (text.length === 0) return;
  if (cachedText(value) === text) return;
  const next = new Map(textCache.value);
  next.set(value, text);
  textCache.value = next;
}

function onNativeChange(event: Event): void {
  const element = nativeSelect();
  if (element === null || items === undefined) return;
  const keys = new Set([...element.selectedOptions].map((option) => option.value));
  const next = items.filter((value) => keys.has(serialize(value)));
  commit(toSelectList<T>(next, true, equals.value), event);
}

function onNativeInvalid(): void {
  userInvalid.value = true;
  triggerElement.value?.focus();
}

function syncNativeSelection(): void {
  const element = nativeSelect();
  if (element === null) return;
  for (const option of element.options) {
    const match = nativeOptions.value.find((entry) => entry.key === option.value);
    option.selected = match?.selected === true;
  }
}

function onFormReset(): void {
  selection.reset();
  userInvalid.value = false;
  void Promise.resolve().then(syncNativeSelection);
}

let observedForm: HTMLFormElement | null = null;

onMounted(() => {
  // Vue mounts children before element props, so option selectedness is
  // re-applied once `multiple` is present on the native select.
  syncNativeSelection();
  observedForm = nativeSelect()?.form ?? null;
  observedForm?.addEventListener("reset", onFormReset);
});

onUnmounted(() => {
  observedForm?.removeEventListener("reset", onFormReset);
  observedForm = null;
});

watch(nativeOptions, syncNativeSelection, { flush: "post" });
watch(
  isOpen,
  (next) => {
    if (next) {
      collection.focusOnOpen(
        nextOpenIntent,
        (value) => includesSelectValue(selected.value, value, equals.value),
        selectedIndex(),
        nextOpenGrapheme,
      );
      return;
    }
    collection.cancelOpenFocus();
    collection.setActiveKey(null);
  },
  { flush: "sync", immediate: true },
);

function anchorElement(): Element | null {
  const options = collection.registry.items.value;
  const selectedOption = options.find((item) =>
    includesSelectValue(selected.value, item.value.value, equals.value),
  );
  return selectedOption?.element ?? options[0]?.element ?? null;
}

const context: SelectContextValue<T> = {
  partPrefix: "select",
  activeDescendant,
  anchorElement,
  baseId,
  activeKey: collection.activeKey,
  choose,
  contentElement,
  direction: directionState,
  disabled: disabledState,
  invalid,
  dismissBranches: computed(() => (triggerElement.value === null ? [] : [triggerElement.value])),
  isItemVisible: () => true,
  isSelected: (value) => includesSelectValue(selected.value, value, equals.value),
  isValueDisabled: (value) => itemDisabled?.(value) === true,
  items: itemsState,
  listboxId,
  multiple: multipleState,
  onTriggerKeydown,
  open: isOpen,
  placeholder: placeholderState,
  listboxLabelledby: triggerId,
  referenceElement: computed(() => triggerElement.value),
  highlight: (key) => collection.setActiveKey(key),
  registerItem: (input) => collection.register(input),
  rememberText,
  required: requiredState,
  selected,
  selectedText,
  setOpen,
  setVirtualAdapter: (adapter) => collection.setVirtualAdapter(adapter),
  state,
  textOf: textFor,
  triggerElement,
  triggerId,
  viewportElement,
};
selectContext.provide(context);

type SelectRootSetupExpose = Omit<
  SelectRootExpose<T>,
  keyof SelectSlotState<T> | "id" | "listboxId" | "triggerId"
> & {
  readonly [Key in keyof SelectSlotState<T>]: ComputedRef<SelectSlotState<T>[Key]>;
} & {
  readonly id: ComputedRef<string>;
  readonly listboxId: ComputedRef<string>;
  readonly triggerId: ComputedRef<string>;
};

const exposed = {
  clear: () => commit(Object.freeze([]), null),
  deselect: (value: T) => commit(removeFromList(selected.value, value, equals.value), null),
  disabled: disabledState,
  focus: (options?: FocusOptions) => triggerElement.value?.focus(options),
  id: baseId,
  invalid,
  listboxId,
  open: isOpen,
  required: requiredState,
  reset: () => selection.reset(),
  select: (value: T) => choose(value, null),
  selected,
  selectedText,
  selectionMode,
  setOpen: (value: boolean, nativeEvent?: Event | null) => setOpen(value, nativeEvent ?? null),
  state,
  triggerId,
} satisfies SelectRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="select"
    part="root"
    :dir="directionState"
    :data-state="state"
    :data-selection-mode="selectionMode"
    :data-disabled="disabledState ? 'true' : undefined"
    :data-required="requiredState ? 'true' : undefined"
    :data-invalid="invalid ? 'true' : undefined"
    :data-empty="selected.length === 0 ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
    <select
      ref="nativeElement"
      data-vize-ui="select-native"
      part="native"
      hidden
      aria-hidden="true"
      tabindex="-1"
      :aria-label="nativeLabel"
      :name
      :form
      :autocomplete
      :required="requiredState"
      :disabled="disabledState"
      :multiple="multipleState"
      @change="onNativeChange"
      @invalid="onNativeInvalid"
    >
      <option v-if="!multipleState" value="" hidden :selected="selected.length === 0">
        {{ nativeLabel }}
      </option>
      <option
        v-for="option in nativeOptions as readonly NativeOption[]"
        :key="option.key"
        :value="option.key"
        :selected="option.selected"
      >
        {{ option.key }}
      </option>
    </select>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
