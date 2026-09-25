<script setup lang="ts" generic="T">
import { computed, onMounted } from "vue";
import type { ComputedRef } from "vue";

import ComboboxRoot from "../combobox/combobox-root.vue";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import type { ComboboxSlotState } from "../combobox/combobox-types.ts";
import { createSelectValueEquality, defaultSelectText } from "../select/select-model.ts";
import { pushAutocompleteHistory, removeAutocompleteHistory } from "./autocomplete-history.ts";
import type {
  AutocompleteRootExpose,
  AutocompleteRootProps,
  AutocompleteSlotState,
} from "./autocomplete-types.ts";

const {
  id = undefined,
  modelValue = undefined,
  defaultValue = undefined,
  items = undefined,
  loadItems = undefined,
  debounce = 200,
  by = undefined,
  itemText = undefined,
  filter = undefined,
  inputValue = undefined,
  history = undefined,
  defaultHistory = undefined,
  maxHistory = 5,
  historyStorage = undefined,
  fromText = undefined,
  openOnFocus = true,
  disabled = false,
  name = undefined,
} = defineProps<AutocompleteRootProps<T>>();

const emit = defineEmits<{
  /** Fired when a suggestion is chosen (or cleared with `null`). */
  "update:modelValue": [value: T | null];

  /** Fired when the input text changes. */
  "update:inputValue": [text: string];

  /** Fired when the recent history changes. */
  "update:history": [history: readonly T[]];

  /** Fired when the user submits: the typed text and the chosen suggestion, if any. */
  submit: [text: string, value: T | null];
}>();

defineSlots<{
  /**
   * Combobox parts. Render `suggestions` as `ComboboxItem`s: history while the
   * query is empty, otherwise the (filtered or loaded) suggestions.
   */
  default?(props: AutocompleteSlotState<T>): unknown;
}>();

const equals = computed(() => createSelectValueEquality<T>(by));
const historyState = useControllableState<readonly T[]>({
  value: () => history,
  defaultValue: () => Object.freeze([...(defaultHistory ?? [])]),
  onChange: (next) => {
    emit("update:history", next);
    historyStorage?.write(next);
  },
});
const recent = computed(() => historyState.value.value);
const historyCount = computed(() => recent.value.length);
const hasHistory = computed(() => historyCount.value > 0);

function currentHistory(): readonly T[] {
  return historyState.value.value;
}

function remember(value: T): void {
  historyState.set(pushAutocompleteHistory(currentHistory(), value, maxHistory, equals.value));
}

function forget(value: T): void {
  historyState.set(removeAutocompleteHistory(currentHistory(), value, equals.value));
}

function clearHistory(): void {
  historyState.set(Object.freeze([]));
}

function onModelValue(value: T | null): void {
  if (value !== null) {
    remember(value);
    emit("submit", itemText?.(value) ?? defaultSelectText(value, by), value);
  }
  emit("update:modelValue", value);
}

function onInputValue(text: string): void {
  emit("update:inputValue", text);
}

// Enter with no highlighted suggestion is a free-text submission. It is
// observed in the capture phase, before the combobox closes its popup.
function onKeydownCapture(event: KeyboardEvent): void {
  if (event.key !== "Enter" || event.isComposing) return;
  if (!(event.target instanceof HTMLInputElement)) return;
  if (event.target.hasAttribute("aria-activedescendant")) return;
  const text = event.target.value.trim();
  if (text.length === 0) return;
  if (fromText !== undefined) remember(fromText(text));
  emit("submit", text, null);
}

onMounted(() => {
  const stored = historyStorage?.read();
  if (stored !== null && stored !== undefined && history === undefined) {
    historyState.set(Object.freeze(stored.slice(0, Math.max(0, maxHistory))));
  }
});

function slotState(state: ComboboxSlotState<T>): AutocompleteSlotState<T> {
  const showingHistory = state.query.length === 0 && recent.value.length > 0;
  return {
    ...state,
    clearHistory,
    forget,
    history: recent.value,
    showingHistory,
    suggestions: showingHistory ? recent.value : state.filteredItems,
  };
}

// Optional props are forwarded only when set, so the combobox keeps its own
// uncontrolled defaults under `exactOptionalPropertyTypes`.
const comboboxProps = computed(() => ({
  ...(id === undefined ? {} : { id }),
  ...(modelValue === undefined ? {} : { modelValue }),
  ...(defaultValue === undefined ? {} : { defaultValue }),
  ...(items === undefined && loadItems === undefined ? { items: recent.value } : {}),
  ...(items === undefined ? {} : { items }),
  ...(loadItems === undefined ? {} : { loadItems }),
  ...(by === undefined ? {} : { by }),
  ...(itemText === undefined ? {} : { itemText }),
  ...(filter === undefined ? {} : { filter }),
  ...(inputValue === undefined ? {} : { inputValue }),
  ...(name === undefined ? {} : { name }),
  onKeydownCapture,
  "onUpdate:inputValue": onInputValue,
  "onUpdate:modelValue": onModelValue,
}));

type AutocompleteRootSetupExpose = Omit<AutocompleteRootExpose<T>, "history"> & {
  readonly history: ComputedRef<readonly T[]>;
};

const exposed = {
  clearHistory,
  forget,
  history: recent,
  remember,
} satisfies AutocompleteRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <ComboboxRoot
    v-bind="comboboxProps"
    :debounce
    :open-on-focus
    :strict="false"
    :disabled
    autocomplete="list"
    data-vize-ui-preset="autocomplete"
    :data-history-count="historyCount"
  >
    <template #default="state: ComboboxSlotState<T>">
      <slot v-bind="slotState(state)" />
    </template>
  </ComboboxRoot>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
