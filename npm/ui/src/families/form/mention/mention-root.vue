<script setup lang="ts" generic="T">
import { computed, nextTick, onUnmounted, shallowRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { useCompositeNavigation } from "../../foundations/composite-navigation/composite-navigation.ts";
import type { CompositeNavigationCommand } from "../../foundations/composite-navigation/composite-navigation.ts";
import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import type { Rect, VirtualElement } from "../../overlays/positioner/positioner.ts";
import {
  measureFieldCaret,
  readFieldCaret,
  readFieldText,
  replaceEditableText,
} from "./mention-caret.ts";
import type { MentionFieldKind } from "./mention-caret.ts";
import { mentionContext } from "./mention-context.ts";
import type { MentionContextValue } from "./mention-context.ts";
import {
  applyMentionEdit,
  containsMentionFilter,
  defaultMentionInsert,
  defaultMentionTriggers,
  detectMention,
  isSameMention,
} from "./mention-core.ts";
import type { MentionMatch, MentionTrigger } from "./mention-core.ts";
import type {
  MentionFilter,
  MentionLoader,
  MentionLoadStatus,
  MentionRootExpose,
  MentionRootProps,
  MentionSlotState,
  MentionState,
} from "./mention-types.ts";

const {
  id = undefined,
  modelValue = undefined,
  defaultValue = "",
  items = undefined,
  loadItems = undefined,
  debounce = 150,
  triggers = defaultMentionTriggers,
  itemText = undefined,
  filter = undefined,
  insert = undefined,
  open = undefined,
  defaultOpen = false,
  loop = false,
  disabled = false,
} = defineProps<MentionRootProps<T>>();

const emit = defineEmits<{
  /** Fired when the field text requests a new controlled value. */
  "update:modelValue": [text: string];

  /** Fired after an item is inserted for the active token, with the item and its trigger. */
  select: [item: T, trigger: MentionTrigger];

  /** Fired when the active query changes; `""` when no token is active. */
  "update:query": [query: string];

  /** Fired when the active token changes, with its query and trigger or `null` for both when it ends. */
  "query-change": [query: string | null, trigger: MentionTrigger | null];

  /** Fired when the popup requests a controlled open value. */
  "update:open": [open: boolean];

  /** Fired after any distinct open-state request with next state, previous state, and event. */
  "open-change": [open: boolean, previous: boolean, nativeEvent: Event | null];
}>();

defineSlots<{
  /** The field (`MentionInput` or `MentionEditable`) and `MentionContent`. Receives query and item state. */
  default?(props: MentionSlotState<T>): unknown;
}>();

/** Data stored in the collection for each rendered item. */
interface MentionRecord {
  readonly id: ComputedRef<string>;
  readonly value: T;
}

const baseId = useDeterministicId({ id: () => id, hint: "mention" });
const fieldId = computed(() => deriveDeterministicId(baseId.value, "field"));
const listboxId = computed(() => deriveDeterministicId(baseId.value, "listbox"));
const disabledState = computed(() => disabled);
const textState = useControllableState<string>({
  value: () => modelValue,
  defaultValue: () => defaultValue,
  onChange: (next) => emit("update:modelValue", next),
});
const text = computed(() => textState.value.value);
const openState = useControllableState({
  value: () => open,
  defaultValue: () => defaultOpen,
});
const match = shallowRef<MentionMatch | null>(null);
const dismissedAt = shallowRef<number | null>(null);
const isOpen = computed(() => openState.value.value && match.value !== null && !disabled);
const state = computed<MentionState>(() => (isOpen.value ? "open" : "closed"));
const query = computed(() => match.value?.query ?? "");
const fieldElement = shallowRef<HTMLElement | null>(null);
const contentElement = shallowRef<HTMLElement | null>(null);
let fieldKind: MentionFieldKind = "text";

const registry = createCollectionRegistry<string, MentionRecord>();
const navigation = useCompositeNavigation({
  registry,
  focusStrategy: "active-descendant",
  getItemId: (item) => item.value.id.value,
  isDisabled: disabledState,
  loop: () => loop,
});
const activeKey = computed(() => registry.activeKey.value);
function currentActiveKey(): string | null {
  return activeKey.value;
}

const activeItem = computed(() => {
  const key = currentActiveKey();
  return key === null ? null : (registry.getItem(key) ?? null);
});
const activeDescendant = computed(() =>
  isOpen.value ? activeItem.value?.value.id.value : undefined,
);
const empty = computed(() => registry.items.value.length === 0);
const triggerChar = computed<string | undefined>(() => match.value?.trigger.char);

const loaded = shallowRef<readonly T[] | undefined>(undefined);
const status = shallowRef<MentionLoadStatus>("idle");
const loadError = shallowRef<unknown>(undefined);

function textFor(item: T): string {
  if (itemText !== undefined) return itemText(item);
  if (typeof item === "string") return item;
  if (typeof item === "object" && item !== null) {
    for (const key of ["label", "name", "title", "text"] as const) {
      const candidate: unknown = Reflect.get(item, key);
      if (typeof candidate === "string") return candidate;
    }
  }
  return String(item);
}

function currentFilter(): MentionFilter<T> | null {
  if (filter === false) return null;
  if (filter !== undefined) return filter;
  return loadItems === undefined
    ? (_item: T, currentQuery: string, _trigger: MentionTrigger, itemTextValue: string) =>
        containsMentionFilter(itemTextValue, currentQuery)
    : null;
}

function currentMatch(): MentionMatch | null {
  return match.value;
}

function currentField(): HTMLElement | null {
  return fieldElement.value;
}

function currentLoader(): MentionLoader<T> | undefined {
  return loadItems;
}

const filteredItems = computed<readonly T[]>(() => {
  const source = (loadItems === undefined ? items : loaded.value) ?? [];
  const active = currentMatch();
  const predicate = currentFilter();
  if (active === null || predicate === null) return source;
  return source.filter((item) => predicate(item, active.query, active.trigger, textFor(item)));
});

function setOpen(value: boolean, nativeEvent: Event | null = null): boolean {
  const previous = openState.value.value;
  const next = value && !disabled;
  const changed = openState.set(next);
  if (changed || previous !== next) {
    emit("update:open", next);
    emit("open-change", next, previous, nativeEvent);
  }
  return changed;
}

function updateMatch(next: MentionMatch | null, nativeEvent: Event | null): void {
  const previous = currentMatch();
  if (next !== null && dismissedAt.value !== null && dismissedAt.value !== next.start) {
    dismissedAt.value = null;
  }
  if (next === null) dismissedAt.value = null;
  if (!isSameMention(previous, next)) {
    match.value = next;
    if (previous?.query !== next?.query || previous?.trigger !== next?.trigger) {
      emit("update:query", next?.query ?? "");
      emit("query-change", next?.query ?? null, next?.trigger ?? null);
    }
  }
  const shouldOpen = next !== null && dismissedAt.value === null;
  if (shouldOpen !== openState.value.value) setOpen(shouldOpen, nativeEvent);
}

function detect(nativeEvent: Event | null): void {
  const element = currentField();
  if (element === null || disabled) {
    updateMatch(null, nativeEvent);
    return;
  }
  const caret = readFieldCaret(element);
  const value = readFieldText(element);
  updateMatch(caret === null ? null : detectMention(value, caret, triggers), nativeEvent);
}

function onFieldInput(event: Event): void {
  const element = currentField();
  if (element === null || disabled) return;
  textState.set(readFieldText(element));
  detect(event);
}

function onFieldCaret(event: Event): void {
  detect(event);
}

function onFieldBlur(event: FocusEvent): void {
  const target = event.relatedTarget;
  if (target instanceof Node && contentElement.value?.contains(target) === true) return;
  if (openState.value.value) setOpen(false, event);
}

function navigate(command: CompositeNavigationCommand, event: Event): void {
  if (registry.activeKey.value === null && (command === "next" || command === "previous")) {
    const key = registry.getNavigationKey(command === "next" ? "first" : "last");
    if (key !== null) registry.setActiveKey(key);
    return;
  }
  navigation.navigate(command, event);
}

function currentActiveItem() {
  return activeItem.value;
}

function chooseActive(event: Event): boolean {
  const item = currentActiveItem();
  if (item === null || item.disabled) return false;
  return choose(item.value.value, event);
}

function dismiss(nativeEvent: Event | null = null): void {
  const active = currentMatch();
  if (active !== null) dismissedAt.value = active.start;
  setOpen(false, nativeEvent);
}

function onFieldKeydown(event: KeyboardEvent): void {
  if (!isOpen.value || event.isComposing || event.defaultPrevented) return;
  switch (event.key) {
    case "ArrowDown":
    case "ArrowUp":
      if (event.altKey || event.ctrlKey || event.metaKey) return;
      event.preventDefault();
      navigate(event.key === "ArrowDown" ? "next" : "previous", event);
      return;
    case "Home":
    case "End":
      event.preventDefault();
      navigate(event.key === "Home" ? "first" : "last", event);
      return;
    case "PageDown":
    case "PageUp":
      event.preventDefault();
      navigate(event.key === "PageDown" ? "page-next" : "page-previous", event);
      return;
    case "Enter":
    case "Tab":
      if (event.shiftKey && event.key === "Tab") return;
      if (chooseActive(event)) event.preventDefault();
      return;
    case "Escape":
      event.preventDefault();
      dismiss(event);
      return;
    default:
  }
}

function choose(value: T, nativeEvent: Event | null): boolean {
  const active = currentMatch();
  const element = currentField();
  if (active === null || disabled) return false;
  const current = element === null ? text.value : readFieldText(element);
  const inserted =
    insert?.(value, active.trigger, textFor(value)) ??
    defaultMentionInsert(textFor(value), active.trigger);
  const edit = applyMentionEdit(current, active, inserted);
  if (element !== null && fieldKind === "editable") {
    replaceEditableText(element, active.start, active.end, inserted);
    textState.set(readFieldText(element));
  } else {
    textState.set(edit.text);
    if (element instanceof HTMLTextAreaElement || element instanceof HTMLInputElement) {
      element.value = edit.text;
      element.setSelectionRange(edit.caret, edit.caret);
    }
  }
  emit("select", value, active.trigger);
  updateMatch(null, nativeEvent);
  element?.focus();
  return true;
}

let controller: AbortController | null = null;
let timer: ReturnType<typeof setTimeout> | null = null;

function cancelLoad(): void {
  controller?.abort();
  controller = null;
  if (timer !== null) clearTimeout(timer);
  timer = null;
}

function runLoad(active: MentionMatch): void {
  const loader = currentLoader();
  if (loader === undefined) return;
  controller?.abort();
  const request = new AbortController();
  controller = request;
  status.value = "loading";
  void Promise.resolve()
    .then(() => loader(active.query, active.trigger, { signal: request.signal }))
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

watch([isOpen, query], ([nextOpen]) => {
  if (loadItems === undefined) return;
  const active = currentMatch();
  if (!nextOpen || active === null) {
    cancelLoad();
    if (status.value === "loading") status.value = "idle";
    return;
  }
  // The previous query is obsolete immediately, even while the next request
  // is waiting for the debounce timer. Its response must never replace the
  // current query's suggestions.
  controller?.abort();
  controller = null;
  loaded.value = undefined;
  loadError.value = undefined;
  status.value = "loading";
  if (timer !== null) clearTimeout(timer);
  timer = null;
  if (debounce <= 0) {
    runLoad(active);
    return;
  }
  timer = setTimeout(() => {
    timer = null;
    const latest = currentMatch();
    if (latest !== null) runLoad(latest);
  }, debounce);
});

watch(query, () => {
  if (registry.activeKey.value !== null) registry.setActiveKey(null);
});

watch(
  [isOpen, registry.navigableItems],
  ([nextOpen, navigable]) => {
    if (!nextOpen) {
      if (registry.activeKey.value !== null) registry.setActiveKey(null);
      return;
    }
    const key = registry.activeKey.value;
    if (key === null || !navigable.some((item) => item.key === key)) {
      registry.setActiveKey(navigable[0]?.key ?? null);
    }
  },
  { flush: "post" },
);

onUnmounted(cancelLoad);

const zeroRect: Rect = Object.freeze({ height: 0, width: 0, x: 0, y: 0 });
const caretReference: VirtualElement = {
  getBoundingClientRect: () => {
    const element = currentField();
    const active = currentMatch();
    if (element === null) return zeroRect;
    if (active === null) return element.getBoundingClientRect();
    return measureFieldCaret(element, active.start);
  },
};

const context: MentionContextValue<T> = {
  activeDescendant,
  activeKey,
  attachField: (element, kind) => {
    fieldElement.value = element;
    fieldKind = kind;
  },
  caretReference,
  choose,
  contentElement,
  disabled: disabledState,
  empty,
  fieldElement,
  fieldId,
  highlight: (key) => {
    if (!registry.navigableItems.value.some((item) => item.key === key)) return false;
    return registry.setActiveKey(key);
  },
  listboxId,
  onFieldBlur,
  onFieldCaret,
  onFieldInput,
  onFieldKeydown,
  open: isOpen,
  query,
  registerItem: (input) =>
    registry.register({
      disabled: input.disabled,
      element: input.element,
      key: input.id.value,
      textValue: input.textValue,
      value: { id: input.id, value: input.value },
    }),
  setOpen,
  state,
  status: computed(() => status.value),
  text,
};
mentionContext.provide(context);

const slotState = computed<MentionSlotState<T>>(() => ({
  error: loadError.value,
  filteredItems: filteredItems.value,
  loading: status.value === "loading",
  match: match.value,
  open: isOpen.value,
  query: query.value,
  state: state.value,
  status: status.value,
  text: text.value,
  trigger: match.value?.trigger ?? null,
}));

type MentionRootSetupExpose = Omit<MentionRootExpose<T>, "match" | "open" | "text"> & {
  readonly match: typeof match;
  readonly open: ComputedRef<boolean>;
  readonly text: ComputedRef<string>;
};

const exposed = {
  dismiss: () => dismiss(null),
  focus: (options?: FocusOptions) => currentField()?.focus(options),
  match,
  open: isOpen,
  refresh: () => {
    void nextTick(() => detect(null));
  },
  select: (item: T) => choose(item, null),
  text,
} satisfies MentionRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="mention"
    part="root"
    :data-state="state"
    :data-trigger="triggerChar"
    :data-disabled="disabledState ? 'true' : undefined"
    :data-loading="status === 'loading' ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
