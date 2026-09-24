import { computed, nextTick, onScopeDispose, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import type {
  CollectionItem,
  CollectionRegistration,
  CollectionRegistry,
} from "../../foundations/collection/collection.ts";
import { useCompositeNavigation } from "../../foundations/composite-navigation/composite-navigation.ts";
import type { CompositeNavigationCommand } from "../../foundations/composite-navigation/composite-navigation.ts";

/**
 * Shared active-descendant option collection for listbox popups.
 *
 * Select and Combobox keep DOM focus on their trigger or input and expose the
 * highlighted option through `aria-activedescendant`. This controller layers
 * the repository's collection registry and composite navigation over option
 * registrations, adds an optional virtualizer adapter so navigation can reach
 * options outside the rendered window, and never touches the DOM during setup.
 */

/** Data stored for each registered option. */
export interface SelectOptionRecord<T> {
  /** Stable DOM id referenced by `aria-activedescendant`. */
  readonly id: ComputedRef<string>;

  /** Consumer-owned option value. */
  readonly value: T;

  /** Absolute index for virtualized options, otherwise `undefined`. */
  readonly index: number | undefined;
}

/** Input accepted when an option registers itself. */
export interface SelectOptionRegistration<T> {
  readonly id: ComputedRef<string>;
  readonly value: T;
  readonly element: MaybeRefOrGetter<Element | null | undefined>;
  readonly textValue: MaybeRefOrGetter<string | null | undefined>;
  readonly disabled: MaybeRefOrGetter<boolean | undefined>;
  readonly index: MaybeRefOrGetter<number | undefined>;
}

/** Bridge installed by a virtualized option list. */
export interface SelectVirtualAdapter {
  /** Total number of options, rendered or not. */
  readonly count: () => number;

  /** Bring the option at `index` into the rendered window. */
  readonly scrollToIndex: (index: number) => void;

  /** Text used for typeahead over options outside the rendered window. */
  readonly textAt?: (index: number) => string;

  /** Whether the option at `index` is disabled. */
  readonly disabledAt?: (index: number) => boolean;
}

/** Where the active option should land when a popup opens. */
export type SelectOpenFocus = "first" | "last" | "selected";

/** Options for {@link useSelectCollection}. */
export interface SelectCollectionOptions {
  /** Wrap arrow navigation at collection boundaries. */
  readonly loop: MaybeRefOrGetter<boolean>;

  /** Suppress navigation entirely. */
  readonly disabled: MaybeRefOrGetter<boolean>;

  /** Enable buffered typeahead over option text. */
  readonly typeahead: MaybeRefOrGetter<boolean>;

  /** Idle time before a typeahead query resets. */
  readonly typeaheadTimeout: MaybeRefOrGetter<number>;

  /** Options traversed by PageUp and PageDown. */
  readonly pageSize?: MaybeRefOrGetter<number>;
}

/** Controller returned by {@link useSelectCollection}. */
export interface SelectCollection<T> {
  readonly registry: CollectionRegistry<string, SelectOptionRecord<T>>;
  readonly activeKey: ComputedRef<string | null>;
  readonly activeItem: ComputedRef<CollectionItem<string, SelectOptionRecord<T>> | null>;
  readonly typeaheadQuery: ComputedRef<string>;
  readonly virtual: Readonly<ShallowRef<SelectVirtualAdapter | null>>;
  readonly register: (input: SelectOptionRegistration<T>) => CollectionRegistration<string>;
  readonly setActiveKey: (key: string | null) => boolean;
  readonly navigate: (command: CompositeNavigationCommand, event?: Event | null) => void;
  readonly typeahead: (event: KeyboardEvent) => boolean;
  readonly typeaheadInput: (grapheme: string) => void;
  readonly activateIndex: (index: number) => void;
  readonly findKey: (predicate: (value: T) => boolean) => string | null;
  readonly focusOnOpen: (
    intent: SelectOpenFocus,
    isSelected: ((value: T) => boolean) | null,
    selectedIndex: number | null,
    grapheme: string | null,
  ) => void;
  readonly cancelOpenFocus: () => void;
  readonly setVirtualAdapter: (adapter: SelectVirtualAdapter | null) => () => void;
}

/** Create the shared option collection in the current component scope. */
export function useSelectCollection<T>(options: SelectCollectionOptions): SelectCollection<T> {
  const registry = createCollectionRegistry<string, SelectOptionRecord<T>>();
  const virtual = shallowRef<SelectVirtualAdapter | null>(null);
  const pendingIndex = shallowRef<number | null>(null);
  const pendingIntent = shallowRef<SelectOpenFocus | null>(null);
  let alive = true;
  onScopeDispose(() => {
    alive = false;
  });
  const pageSize = () => Math.max(1, Math.trunc(toValue(options.pageSize) ?? 10));

  const navigation = useCompositeNavigation({
    registry,
    focusStrategy: "active-descendant",
    getItemId: (item) => item.value.id.value,
    loop: () => toValue(options.loop),
    isDisabled: () => toValue(options.disabled),
    pageSize,
    typeahead: {
      isDisabled: () => !toValue(options.typeahead),
      timeout: () => toValue(options.typeaheadTimeout),
    },
    scrollIntoView: (item) => {
      const adapter = virtual.value;
      if (adapter !== null && item.value.index !== undefined) {
        adapter.scrollToIndex(item.value.index);
        return;
      }
      const element = item.element as Partial<HTMLElement> | null;
      if (typeof element?.scrollIntoView === "function") {
        element.scrollIntoView({ block: "nearest", inline: "nearest" });
      }
    },
  });

  const activeKey = computed(() => registry.activeKey.value);
  const activeItem = computed(() => {
    const key = activeKey.value;
    return key === null ? null : (registry.getItem(key) ?? null);
  });
  const typeaheadQuery = computed(() => navigation.typeahead?.query.value ?? "");

  function findKeyByIndex(index: number): string | null {
    const match = registry.navigableItems.value.find((item) => item.value.index === index);
    return match?.key ?? null;
  }

  function setActiveKey(key: string | null): boolean {
    if (key !== null && !registry.navigableItems.value.some((item) => item.key === key)) {
      return false;
    }
    return registry.setActiveKey(key);
  }

  function nextEnabledIndex(start: number, step: 1 | -1): number | null {
    const adapter = virtual.value;
    if (adapter === null) return null;
    const count = adapter.count();
    for (let index = start; index >= 0 && index < count; index += step) {
      if (adapter.disabledAt?.(index) !== true) return index;
    }
    return null;
  }

  function activateIndex(index: number): void {
    const key = findKeyByIndex(index);
    if (key !== null) {
      setActiveKey(key);
      virtual.value?.scrollToIndex(index);
      return;
    }
    pendingIndex.value = index;
    virtual.value?.scrollToIndex(index);
  }

  function applyPending(): void {
    const index = pendingIndex.value;
    if (index !== null) {
      const key = findKeyByIndex(index);
      if (key !== null) {
        pendingIndex.value = null;
        setActiveKey(key);
      }
    }
  }

  function navigate(command: CompositeNavigationCommand, event: Event | null = null): void {
    if (toValue(options.disabled)) return;
    const adapter = virtual.value;
    const current = activeItem.value?.value.index;
    if (adapter !== null && (command === "first" || command === "last")) {
      const target =
        command === "first" ? nextEnabledIndex(0, 1) : nextEnabledIndex(adapter.count() - 1, -1);
      if (target !== null) activateIndex(target);
      return;
    }
    if (adapter !== null && (command === "page-next" || command === "page-previous")) {
      const count = adapter.count();
      const base = current ?? (command === "page-next" ? -1 : count);
      const step = command === "page-next" ? 1 : -1;
      const raw = Math.min(count - 1, Math.max(0, base + step * pageSize()));
      const target = nextEnabledIndex(raw, step) ?? nextEnabledIndex(raw, step === 1 ? -1 : 1);
      if (target !== null) activateIndex(target);
      return;
    }
    if (registry.activeKey.value === null && (command === "next" || command === "previous")) {
      const key = registry.getNavigationKey(command === "next" ? "first" : "last");
      if (key !== null) {
        setActiveKey(key);
        return;
      }
    }
    navigation.navigate(command, event);
  }

  function virtualTypeahead(query: string): boolean {
    const adapter = virtual.value;
    if (adapter?.textAt === undefined) return false;
    const count = adapter.count();
    const normalized = query.toLocaleLowerCase();
    const start = (activeItem.value?.value.index ?? -1) + (query.length === 1 ? 1 : 0);
    for (let offset = 0; offset < count; offset++) {
      const index = (start + offset + count) % count;
      if (adapter.disabledAt?.(index) === true) continue;
      if (adapter.textAt(index).toLocaleLowerCase().startsWith(normalized)) {
        activateIndex(index);
        return true;
      }
    }
    return false;
  }

  function typeahead(event: KeyboardEvent): boolean {
    const controller = navigation.typeahead;
    if (controller === null || !toValue(options.typeahead) || toValue(options.disabled)) {
      return false;
    }
    if (event.key.length === 0 || event.altKey || event.ctrlKey || event.metaKey) return false;
    const before = controller.query.value;
    controller.typeaheadProps.onKeydown(event);
    if (!event.defaultPrevented) return false;
    const query = controller.query.value;
    if (virtual.value !== null && query.length > 0 && query !== before) virtualTypeahead(query);
    return true;
  }

  function typeaheadInput(grapheme: string): void {
    const controller = navigation.typeahead;
    if (controller === null || !toValue(options.typeahead)) return;
    controller.input(grapheme);
    if (virtual.value !== null) virtualTypeahead(controller.query.value);
  }

  function findKey(predicate: (value: T) => boolean): string | null {
    return registry.navigableItems.value.find((item) => predicate(item.value.value))?.key ?? null;
  }

  let pendingSelected: ((value: T) => boolean) | null = null;
  let pendingGrapheme: string | null = null;
  let pendingSelectedIndex: number | null = null;

  function applyIntent(intent: SelectOpenFocus): boolean {
    if (virtual.value !== null && pendingGrapheme === null) {
      if (intent === "selected" && pendingSelectedIndex !== null) {
        activateIndex(pendingSelectedIndex);
        return true;
      }
      if (intent !== "selected") {
        navigate(intent);
        return true;
      }
    }
    if (registry.navigableItems.value.length === 0) return false;
    if (intent === "selected" && pendingSelected !== null) {
      const key = findKey(pendingSelected);
      if (key !== null) {
        setActiveKey(key);
        return true;
      }
    }
    const key = registry.getNavigationKey(intent === "last" ? "last" : "first");
    if (key !== null) setActiveKey(key);
    return true;
  }

  function flushIntent(): void {
    const intent = pendingIntent.value;
    if (intent === null || !alive) return;
    if (applyIntent(intent)) {
      const grapheme = pendingGrapheme;
      pendingIntent.value = null;
      pendingSelected = null;
      pendingGrapheme = null;
      pendingSelectedIndex = null;
      if (grapheme !== null) typeaheadInput(grapheme);
    }
  }

  function focusOnOpen(
    intent: SelectOpenFocus,
    isSelected: ((value: T) => boolean) | null,
    selectedIndex: number | null,
    grapheme: string | null,
  ): void {
    const adapter = virtual.value;
    if (adapter !== null && grapheme === null) {
      if (intent === "selected" && selectedIndex !== null) {
        activateIndex(selectedIndex);
        return;
      }
      if (intent !== "selected") {
        navigate(intent);
        return;
      }
    }
    pendingIntent.value = intent;
    pendingSelected = isSelected;
    pendingGrapheme = grapheme;
    pendingSelectedIndex = selectedIndex;
    // Options register during the popup's mount; wait until the whole batch
    // is present so a later-registered selected option wins over the first.
    void nextTick(flushIntent);
  }

  function cancelOpenFocus(): void {
    pendingIntent.value = null;
    pendingSelected = null;
    pendingGrapheme = null;
    pendingSelectedIndex = null;
    pendingIndex.value = null;
  }

  watch(registry.navigableItems, applyPending, { flush: "sync" });
  watch(registry.navigableItems, flushIntent, { flush: "post" });

  function setVirtualAdapter(adapter: SelectVirtualAdapter | null): () => void {
    virtual.value = adapter;
    return () => {
      if (virtual.value === adapter) virtual.value = null;
    };
  }

  function register(input: SelectOptionRegistration<T>): CollectionRegistration<string> {
    return registry.register({
      key: input.id.value,
      value: {
        id: input.id,
        value: input.value,
        get index() {
          return toValue(input.index);
        },
      },
      element: input.element,
      textValue: input.textValue,
      disabled: input.disabled,
      order: input.index,
    });
  }

  return Object.freeze({
    registry,
    activeKey,
    activeItem,
    typeaheadQuery,
    virtual,
    register,
    setActiveKey,
    navigate,
    typeahead,
    typeaheadInput,
    activateIndex,
    findKey,
    focusOnOpen,
    cancelOpenFocus,
    setVirtualAdapter,
  }) satisfies SelectCollection<T>;
}
