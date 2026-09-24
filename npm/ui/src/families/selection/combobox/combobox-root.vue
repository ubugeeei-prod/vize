<script setup lang="ts" generic="T, Multiple extends boolean = false">
import { computed, nextTick, onMounted, onUnmounted, shallowRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import VisuallyHidden from "../../accessibility/visually-hidden/visually-hidden.vue";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { useSelectCollection } from "../select/select-collection.ts";
import { useSelectSelection } from "../select/select-selection.ts";
import type { SelectOpenFocus } from "../select/select-collection.ts";
import { selectContext } from "../select/select-context.ts";
import type { SelectContextValue } from "../select/select-context.ts";
import { readBooleanProp } from "../select/select-keyboard.ts";
import {
  createSelectValueEquality,
  defaultSelectText,
  fromSelectList,
  includesSelectValue,
  removeFromList,
  serializeSelectValue,
  toggleInList,
} from "../select/select-model.ts";
import type { SelectModelValue } from "../select/select-model.ts";
import { comboboxContext } from "./combobox-context.ts";
import type { ComboboxContextValue } from "./combobox-context.ts";
import {
  containsComboboxFilter,
  inlineComboboxCompletion,
  normalizeComboboxText,
} from "./combobox-filter.ts";
import type {
  ComboboxFilter,
  ComboboxLoader,
  ComboboxLoadStatus,
  ComboboxRootExpose,
  ComboboxRootProps,
  ComboboxSelectionMode,
  ComboboxSlotState,
  ComboboxState,
} from "./combobox-types.ts";

const {
  id = undefined,
  modelValue = undefined,
  defaultValue = undefined,
  multiple = undefined,
  items = undefined,
  loadItems = undefined,
  debounce = 200,
  by = undefined,
  itemText = undefined,
  itemDisabled = undefined,
  formValue = undefined,
  filter = undefined,
  autocomplete = "list",
  strict = true,
  createOption = undefined,
  inputValue = undefined,
  defaultInputValue = undefined,
  open = undefined,
  defaultOpen = false,
  openOnFocus = false,
  autoHighlight = true,
  closeOnSelect = undefined,
  clearOnEscape = true,
  loop = false,
  disabled = false,
  readonly = false,
  required = false,
  name = undefined,
  form = undefined,
  dir = "ltr",
  ariaInvalid = false,
} = defineProps<ComboboxRootProps<T, Multiple>>();

const emit = defineEmits<{
  /** Fired when the selection requests a new controlled value (`T | null` or `readonly T[]`). */
  "update:modelValue": [value: SelectModelValue<T, Multiple>];

  /** Fired after a user interaction changes the selection: next value, previous value, and the native event. */
  change: [
    value: SelectModelValue<T, Multiple>,
    previous: SelectModelValue<T, Multiple>,
    nativeEvent: Event,
  ];

  /** Fired when the input text requests a new controlled value. */
  "update:inputValue": [text: string];

  /** Fired when the popup requests a controlled open value. */
  "update:open": [open: boolean];

  /** Fired after any distinct open-state request with the next state, previous state, and triggering event. */
  "open-change": [open: boolean, previous: boolean, nativeEvent: Event | null];

  /** Fired when `createOption` builds a new value from the typed text, before it is selected. */
  create: [value: T, text: string];
}>();

defineSlots<{
  /** Compound Combobox parts. Receives selection, text, filtered items, and loader state. */
  default?(props: ComboboxSlotState<T>): unknown;
}>();

/** Option entries stored in the shared collection: real values or the create option. */
type ComboboxEntry = { readonly kind: "value"; readonly value: T } | { readonly kind: "create" };

/** One hidden form input. */
interface FormEntry {
  readonly key: string;
  readonly value: string;
}

const baseId = useDeterministicId({ id: () => id, hint: "combobox" });
const inputId = computed(() => deriveDeterministicId(baseId.value, "input"));
const listboxId = computed(() => deriveDeterministicId(baseId.value, "listbox"));
const multipleState = computed(() => readBooleanProp(multiple));
const disabledState = computed(() => disabled);
const readonlyState = computed(() => readonly);
const requiredState = computed(() => required);
const autocompleteState = computed(() => autocomplete);
const equals = computed(() => createSelectValueEquality<T>(by));
const userInvalid = shallowRef(false);
const query = shallowRef("");
const labelledby = shallowRef<string | undefined>(undefined);

const selection = useSelectSelection<T, Multiple>({
  defaultValue: () => defaultValue,
  equals: () => equals.value,
  modelValue: () => modelValue,
  multiple: () => multipleState.value,
  onChange: (next) => emit("update:modelValue", next),
});
const selected = computed(() => selection.value.value);

const collection = useSelectCollection<ComboboxEntry>({
  disabled: disabledState,
  loop: () => loop,
  typeahead: () => false,
  typeaheadTimeout: () => 500,
});
const textCache = shallowRef<ReadonlyMap<T, string>>(new Map());

function serialize(value: T): string {
  return formValue?.(value) ?? serializeSelectValue(value, by);
}

function registeredText(value: T): string | undefined {
  for (const item of collection.registry.items.value) {
    const entry = item.value.value;
    if (entry.kind === "value" && equals.value(entry.value, value)) {
      return item.textValue.length > 0 ? item.textValue : undefined;
    }
  }
  return undefined;
}

function textFor(value: T): string {
  return (
    itemText?.(value) ?? cachedText(value) ?? registeredText(value) ?? defaultSelectText(value, by)
  );
}

function cachedText(value: T): string | undefined {
  for (const [candidate, text] of textCache.value) {
    if (equals.value(candidate, value)) return text;
  }
  return undefined;
}

function selectionLabel(): string {
  const first = selection.value.value[0];
  return multipleState.value || first === undefined ? "" : textFor(first);
}

const textState = useControllableState<string>({
  value: () => inputValue,
  defaultValue: () => defaultInputValue ?? selectionLabel(),
  onChange: (next) => emit("update:inputValue", next),
});
const text = computed(() => textState.value.value);

const openState = useControllableState({
  value: () => open,
  defaultValue: () => defaultOpen,
});
const isOpen = computed(() => openState.value.value && !disabledState.value);
const state = computed<ComboboxState>(() => (isOpen.value ? "open" : "closed"));
const invalid = computed(() => ariaInvalid !== false || userInvalid.value);
const selectionMode = computed<ComboboxSelectionMode>(() =>
  multipleState.value ? "multiple" : "single",
);

const loaded = shallowRef<readonly T[] | undefined>(undefined);
const status = shallowRef<ComboboxLoadStatus>("idle");
const loadError = shallowRef<unknown>(undefined);
const sourceItems = computed(() => (loadItems === undefined ? items : loaded.value));
const filters = computed(() => autocomplete === "list" || autocomplete === "both");
const filterFunction = computed(() => {
  if (filter === false || !filters.value) return null;
  return filter ?? (loadItems === undefined ? containsComboboxFilter : null);
});
const filteredItems = computed<readonly T[]>(() => {
  const source = sourceItems.value ?? [];
  const match = currentFilter();
  if (match === null || query.value.length === 0) return source;
  return source.filter((value) => match(value, query.value, textFor(value)));
});
const visibleCount = computed(
  () => collection.registry.items.value.filter((item) => item.value.value.kind === "value").length,
);
const empty = computed(() => visibleCount.value === 0);
const activeEntry = computed(() => collection.activeItem.value?.value.value ?? null);
const activeDescendant = computed(() =>
  isOpen.value ? collection.activeItem.value?.value.id.value : undefined,
);
const exactMatch = computed(() => {
  const needle = normalizeComboboxText(query.value);
  if (needle.length === 0) return false;
  const source = currentSource();
  if (source !== undefined) {
    return source.some((value) => normalizeComboboxText(textFor(value)) === needle);
  }
  return collection.registry.items.value.some(
    (item) => item.value.value.kind === "value" && normalizeComboboxText(item.textValue) === needle,
  );
});
const canCreate = computed(
  () =>
    !disabledState.value &&
    !readonlyState.value &&
    createOption !== undefined &&
    query.value.trim().length > 0 &&
    !exactMatch.value,
);
const createActive = computed(() => activeEntry.value?.kind === "create");
const selectedText = computed(() => selected.value.map(textFor));
const formEntries = computed<readonly FormEntry[]>(() => {
  if (selected.value.length === 0) {
    const fallback = !strict ? text.value : "";
    return multipleState.value && fallback === "" ? [] : [{ key: "empty", value: fallback }];
  }
  return selected.value.map((value, index) => ({ key: `${index}`, value: serialize(value) }));
});
const nativeRequired = computed(() => requiredState.value && selected.value.length === 0);
const loadAnnouncement = computed(() => {
  if (status.value === "loading") return "Loading…";
  if (status.value === "success") return `${filteredItems.value.length} results available.`;
  if (status.value === "error") return "Results could not be loaded.";
  return "";
});

const inputElement = shallowRef<HTMLInputElement | null>(null);
const anchorElement = shallowRef<HTMLElement | null>(null);
const toggleElement = shallowRef<HTMLElement | null>(null);
const contentElement = shallowRef<HTMLElement | null>(null);
const viewportElement = shallowRef<HTMLElement | null>(null);

function currentFilter(): ComboboxFilter<T> | null {
  return filterFunction.value;
}

function currentSource(): readonly T[] | undefined {
  return sourceItems.value;
}

function currentLoader(): ComboboxLoader<T> | undefined {
  return loadItems;
}

function currentQueryText(): string {
  return query.value;
}

function currentSelection(): readonly T[] {
  return selection.value.value;
}

function currentOpen(): boolean {
  return isOpen.value;
}

function input(): HTMLInputElement | null {
  return inputElement.value;
}

function syncRequiredValidity(): void {
  const element = input();
  if (element === null) return;
  const unselectedText =
    requiredState.value && strict && selected.value.length === 0 && element.value.trim().length > 0;
  element.setCustomValidity(unselectedText ? "Select an option from the list." : "");
}

function setText(next: string): void {
  textState.set(next);
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

let nextOpenIntent: SelectOpenFocus | null = "selected";

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

function openWith(intent: SelectOpenFocus | null, nativeEvent: Event | null): void {
  nextOpenIntent = intent;
  setOpen(true, nativeEvent);
  nextOpenIntent = "selected";
}

function isSelectedEntry(entry: ComboboxEntry): boolean {
  return entry.kind === "value" && includesSelectValue(selected.value, entry.value, equals.value);
}

watch(
  isOpen,
  (next) => {
    if (next) {
      const intent = nextOpenIntent;
      if (intent !== null) collection.focusOnOpen(intent, isSelectedEntry, null, null);
      if (loadItems !== undefined && loaded.value === undefined) scheduleLoad(true);
      return;
    }
    collection.cancelOpenFocus();
    collection.setActiveKey(null);
  },
  { flush: "sync" },
);

function choose(value: T, nativeEvent: Event | null): boolean {
  if (disabledState.value || readonlyState.value) return false;
  const changed = commit(
    toggleInList(currentSelection(), value, multipleState.value, equals.value),
    nativeEvent,
  );
  query.value = "";
  setText(multipleState.value ? "" : textFor(value));
  if (closeOnSelect ?? !multipleState.value) setOpen(false, nativeEvent);
  return changed;
}

function create(nativeEvent: Event | null): boolean {
  const typed = query.value.trim();
  if (
    disabledState.value ||
    readonlyState.value ||
    createOption === undefined ||
    typed.length === 0
  ) {
    return false;
  }
  const value = createOption(typed);
  emit("create", value, typed);
  return choose(value, nativeEvent);
}

function remove(value: T, nativeEvent: Event | null = null): boolean {
  if (disabledState.value || readonlyState.value) return false;
  return commit(removeFromList(currentSelection(), value, equals.value), nativeEvent);
}

function chooseActive(nativeEvent: Event): boolean {
  const item = collection.activeItem.value;
  if (item === null || item.disabled) return false;
  const entry = item.value.value;
  if (entry.kind === "create") create(nativeEvent);
  else choose(entry.value, nativeEvent);
  return true;
}

function syncTextToSelection(): void {
  query.value = "";
  setText(selectionLabel());
}

function highlightFirst(): void {
  if (!autoHighlight) return;
  collection.focusOnOpen("first", null, null, null);
}

function completeInline(typed: string): void {
  void nextTick(() => {
    if (query.value !== typed) return;
    for (const item of collection.registry.navigableItems.value) {
      const entry = item.value.value;
      if (entry.kind !== "value") continue;
      const completion = inlineComboboxCompletion(typed, textFor(entry.value));
      if (completion === null) continue;
      collection.setActiveKey(item.key);
      setText(completion);
      void nextTick(() => input()?.setSelectionRange(typed.length, completion.length));
      return;
    }
  });
}

function onInput(event: Event): void {
  const target = event.target;
  if (!(target instanceof HTMLInputElement) || disabledState.value || readonlyState.value) return;
  const typed = target.value;
  query.value = typed;
  setText(typed);
  if (typed.length === 0 && !multipleState.value && strict) commit(Object.freeze([]), event);
  syncRequiredValidity();
  if (!isOpen.value) openWith(null, event);
  const deleting = event instanceof InputEvent && event.inputType.startsWith("delete");
  if ((autocomplete === "inline" || autocomplete === "both") && !deleting) {
    completeInline(typed);
    return;
  }
  if (autocomplete !== "none") highlightFirst();
}

function onKeydown(event: KeyboardEvent): void {
  if (disabledState.value || event.defaultPrevented || event.isComposing) return;
  const { key } = event;
  switch (key) {
    case "ArrowDown":
      if (readonlyState.value) return;
      event.preventDefault();
      if (!isOpen.value) {
        openWith(event.altKey ? null : "selected", event);
        return;
      }
      collection.navigate("next", event);
      return;
    case "ArrowUp":
      if (readonlyState.value) return;
      event.preventDefault();
      if (!isOpen.value) {
        openWith("last", event);
        return;
      }
      if (event.altKey) {
        setOpen(false, event);
        return;
      }
      collection.navigate("previous", event);
      return;
    case "PageDown":
    case "PageUp":
      if (!isOpen.value) return;
      event.preventDefault();
      collection.navigate(key === "PageDown" ? "page-next" : "page-previous", event);
      return;
    case "Enter":
      if (isOpen.value && collection.activeItem.value !== null) {
        event.preventDefault();
        chooseActive(event);
        return;
      }
      if (canCreate.value) {
        event.preventDefault();
        create(event);
        return;
      }
      if (isOpen.value) {
        event.preventDefault();
        if (strict) syncTextToSelection();
        setOpen(false, event);
      }
      return;
    case "Escape":
      if (isOpen.value) {
        event.preventDefault();
        setOpen(false, event);
        return;
      }
      if (
        clearOnEscape &&
        !readonlyState.value &&
        (text.value !== "" || (!multipleState.value && selected.value.length > 0))
      ) {
        event.preventDefault();
        query.value = "";
        setText("");
        if (!multipleState.value) commit(Object.freeze([]), event);
      }
      return;
    case "Backspace": {
      const element = input();
      const last = selected.value.at(-1);
      if (
        multipleState.value &&
        !readonlyState.value &&
        text.value === "" &&
        last !== undefined &&
        (element === null || element.selectionStart === 0)
      ) {
        event.preventDefault();
        commit(removeFromList(currentSelection(), last, equals.value), event);
      }
      return;
    }
    case "Tab":
      if (isOpen.value) setOpen(false, event);
      return;
    default:
  }
}

function onFocus(event: FocusEvent): void {
  if (openOnFocus && !readonlyState.value && !isOpen.value) openWith("selected", event);
}

function isInside(target: EventTarget | null): boolean {
  if (!(target instanceof Node)) return false;
  return [contentElement.value, anchorElement.value, toggleElement.value].some(
    (element) => element?.contains(target) === true,
  );
}

function onBlur(event: FocusEvent): void {
  if (isInside(event.relatedTarget)) return;
  if (strict) syncTextToSelection();
  else query.value = "";
}

let controller: AbortController | null = null;
let timer: ReturnType<typeof setTimeout> | null = null;
let mounted = false;

function runLoad(currentQuery: string): void {
  const loader = currentLoader();
  if (loader === undefined) return;
  controller?.abort();
  const request = new AbortController();
  controller = request;
  status.value = "loading";
  void Promise.resolve()
    .then(() => loader(currentQuery, { signal: request.signal }))
    .then(
      (result) => {
        if (request.signal.aborted) return;
        loaded.value = Object.freeze([...result]);
        loadError.value = undefined;
        status.value = "success";
      },
      (error: unknown) => {
        if (request.signal.aborted) return;
        loadError.value = error;
        status.value = "error";
      },
    );
}

function scheduleLoad(immediate: boolean): void {
  if (loadItems === undefined || !mounted) return;
  if (timer !== null) clearTimeout(timer);
  timer = null;
  // A request for the previous query is stale as soon as this query arrives,
  // even when the replacement waits for the debounce interval.
  controller?.abort();
  const currentQuery = currentQueryText();
  if (immediate || debounce <= 0) {
    runLoad(currentQuery);
    return;
  }
  timer = setTimeout(() => {
    timer = null;
    runLoad(currentQuery);
  }, debounce);
}

watch(query, () => scheduleLoad(false));
watch([inputElement, text, selected, requiredState], syncRequiredValidity, {
  flush: "post",
  immediate: true,
});
watch(filteredItems, () => {
  if (isOpen.value && query.value.length > 0 && autocomplete !== "none") highlightFirst();
});
watch(selected, () => {
  if (!multipleState.value && query.value === "") setText(selectionLabel());
});

function rememberText(value: T, next: string): void {
  if (next.length === 0) return;
  if (cachedText(value) === next) return;
  const map = new Map(textCache.value);
  map.set(value, next);
  textCache.value = map;
}

function onFormReset(): void {
  selection.reset();
  query.value = "";
  userInvalid.value = false;
  setText(defaultInputValue ?? selectionLabel());
}

let observedForm: HTMLFormElement | null = null;

function onInvalid(): void {
  userInvalid.value = true;
}

onMounted(() => {
  mounted = true;
  observedForm = input()?.form ?? null;
  observedForm?.addEventListener("reset", onFormReset);
  input()?.addEventListener("invalid", onInvalid);
  if (isOpen.value && loadItems !== undefined) scheduleLoad(true);
});

onUnmounted(() => {
  mounted = false;
  controller?.abort();
  if (timer !== null) clearTimeout(timer);
  observedForm?.removeEventListener("reset", onFormReset);
  input()?.removeEventListener("invalid", onInvalid);
});

const popupContext: SelectContextValue<T> = {
  activeDescendant,
  activeKey: collection.activeKey,
  anchorElement: () => null,
  baseId,
  choose,
  contentElement,
  direction: computed(() => dir),
  disabled: disabledState,
  dismissBranches: computed(() =>
    [inputElement.value, anchorElement.value, toggleElement.value].filter(
      (element): element is HTMLElement => element !== null,
    ),
  ),
  highlight: (key) => collection.setActiveKey(key),
  invalid,
  isItemVisible: (value, itemTextValue) => {
    const match = currentFilter();
    if (sourceItems.value !== undefined || match === null || query.value.length === 0) return true;
    return match(value, query.value, itemTextValue());
  },
  isSelected: (value) => includesSelectValue(selected.value, value, equals.value),
  isValueDisabled: (value) => itemDisabled?.(value) === true,
  items: filteredItems,
  listboxId,
  listboxLabelledby: computed(() => labelledby.value),
  multiple: multipleState,
  onTriggerKeydown: onKeydown,
  open: isOpen,
  partPrefix: "combobox",
  placeholder: computed(() => undefined),
  referenceElement: computed(() => anchorElement.value ?? inputElement.value),
  registerItem: (registration) =>
    collection.register({
      ...registration,
      value: { kind: "value", value: registration.value },
    }),
  rememberText,
  required: requiredState,
  selected,
  selectedText,
  setOpen,
  setVirtualAdapter: (adapter) => collection.setVirtualAdapter(adapter),
  state,
  textOf: textFor,
  triggerElement: inputElement,
  triggerId: inputId,
  viewportElement,
};
selectContext.provide(popupContext);

const context: ComboboxContextValue<T> = {
  activeDescendant,
  anchorElement,
  autocomplete: autocompleteState,
  canCreate,
  create,
  createActive,
  disabled: disabledState,
  empty,
  highlightCreateOption: () => {
    const key = collection.findKey((entry) => entry.kind === "create");
    if (key !== null) collection.setActiveKey(key);
  },
  inputElement,
  inputId,
  inputValue: text,
  invalid,
  isValueDisabled: (value) => itemDisabled?.(value) === true,
  labelledby,
  listboxId,
  multiple: multipleState,
  nativeRequired,
  onBlur,
  onFocus,
  onInput,
  onKeydown,
  open: isOpen,
  query: computed(() => query.value),
  readonly: readonlyState,
  registerCreateOption: ({ element, id: optionId }) =>
    collection.register({
      disabled: false,
      element,
      id: optionId,
      index: undefined,
      textValue: "",
      value: { kind: "create" },
    }),
  remove,
  required: requiredState,
  setOpen,
  status: computed(() => status.value),
  textOf: textFor,
  toggleElement,
};
comboboxContext.provide(context);

const slotState = computed<ComboboxSlotState<T>>(() => ({
  disabled: disabledState.value,
  empty: empty.value,
  error: loadError.value,
  filteredItems: filteredItems.value,
  inputValue: text.value,
  loading: status.value === "loading",
  open: isOpen.value,
  query: query.value,
  remove,
  selected: selected.value,
  selectedText: selectedText.value,
  selectionMode: selectionMode.value,
  state: state.value,
  status: status.value,
}));

type ComboboxRootSetupExpose = Omit<ComboboxRootExpose<T>, "inputValue" | "open" | "selected"> & {
  readonly inputValue: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly selected: ComputedRef<readonly T[]>;
};

const exposed = {
  clear: () => {
    query.value = "";
    setText("");
    return commit(Object.freeze([]), null);
  },
  deselect: remove,
  focus: (options?: FocusOptions) => input()?.focus(options),
  inputValue: text,
  open: isOpen,
  reload: () => scheduleLoad(true),
  reset: onFormReset,
  select: (value: T) => choose(value, null),
  selected,
  setInputValue: (next: string) => {
    query.value = next;
    setText(next);
  },
  setOpen: (value: boolean, nativeEvent?: Event | null) => setOpen(value, nativeEvent ?? null),
} satisfies ComboboxRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="combobox"
    part="root"
    :dir
    :data-state="state"
    :data-selection-mode="selectionMode"
    :data-autocomplete="autocompleteState"
    :data-disabled="disabledState ? 'true' : undefined"
    :data-readonly="readonlyState ? 'true' : undefined"
    :data-required="requiredState ? 'true' : undefined"
    :data-invalid="invalid ? 'true' : undefined"
    :data-empty="selected.length === 0 ? 'true' : undefined"
    :data-loading="status === 'loading' ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
    <VisuallyHidden v-if="loadItems !== undefined" role="status" aria-live="polite">
      {{ loadAnnouncement }}
    </VisuallyHidden>
    <template v-if="name !== undefined">
      <input
        v-for="entry in formEntries as readonly FormEntry[]"
        :key="entry.key"
        type="hidden"
        data-vize-ui="combobox-native"
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
