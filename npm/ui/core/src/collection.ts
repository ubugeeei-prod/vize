import { computed, getCurrentScope, onScopeDispose, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

/** Serializable identity accepted by every collection primitive. */
export type CollectionKey = string | number;

/** How disabled items participate in focus navigation and typeahead. */
export type CollectionDisabledBehavior = "skip" | "focusable";

/** Direction understood by {@link CollectionRegistry.getNavigationKey}. */
export type CollectionNavigationDirection = "first" | "last" | "next" | "previous";

/** Why the registry changed its active key. */
export type CollectionActiveChangeReason =
  | "programmatic"
  | "navigation"
  | "typeahead"
  | "item-removed"
  | "item-disabled"
  | "registry-disposed";

/** Reactive sources registered for one logical collection item. */
export interface CollectionItemInput<Key extends CollectionKey, Value> {
  /** Stable, request-independent identity. Keys must be unique within a registry. */
  readonly key: Key;

  /** Consumer-owned data associated with this item. */
  readonly value: Value;

  /** Rendered element used for live DOM ordering and text extraction. */
  readonly element?: MaybeRefOrGetter<Element | null | undefined>;

  /**
   * Explicit typeahead text. `null` and `undefined` select accessible DOM text
   * extraction; an empty string deliberately opts the item out of typeahead.
   */
  readonly textValue?: MaybeRefOrGetter<string | null | undefined>;

  /** Whether activation is disabled. Navigation follows `disabledBehavior`. */
  readonly disabled?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Deterministic order for virtualized, portalled, or server-only items.
   *
   * If one item supplies an order, every item must supply a unique safe integer.
   * Otherwise DOM order is used once every item is connected in one document;
   * registration order is the SSR, partial-mount, and disconnected fallback.
   */
  readonly order?: MaybeRefOrGetter<number | undefined>;
}

/** Immutable, resolved view of a registered collection item. */
export interface CollectionItem<Key extends CollectionKey, Value> {
  /** Stable item identity. */
  readonly key: Key;

  /** Consumer-owned item data. */
  readonly value: Value;

  /** Current rendered element, or `null` during SSR and before mount. */
  readonly element: Element | null;

  /** Normalized text used by typeahead. */
  readonly textValue: string;

  /** Current disabled state. */
  readonly disabled: boolean;

  /** Explicit virtual order, or `undefined` when DOM order is authoritative. */
  readonly order: number | undefined;
}

/** Active-key transition reported to collection consumers. */
export interface CollectionActiveChange<Key extends CollectionKey> {
  /** New active key, or `null` when no navigable item remains. */
  readonly key: Key | null;

  /** Active key before the transition. */
  readonly previousKey: Key | null;

  /** Stable cause suitable for focus and announcement policies. */
  readonly reason: CollectionActiveChangeReason;
}

/** Options for {@link createCollectionRegistry}. */
export interface CollectionRegistryOptions<Key extends CollectionKey> {
  /**
   * Whether disabled items are omitted from navigation or remain focusable.
   *
   * Menus commonly use `"focusable"`; listboxes and roving tab stops commonly
   * use `"skip"`.
   *
   * @default "skip"
   */
  readonly disabledBehavior?: CollectionDisabledBehavior;

  /** Locale-aware comparator used by typeahead matching. */
  readonly collator?: Intl.Collator;

  /** Called synchronously after a distinct active-key transition. */
  readonly onActiveKeyChange?: (change: CollectionActiveChange<Key>) => void;
}

/** Options for directional collection navigation. */
export interface CollectionNavigationOptions<Key extends CollectionKey> {
  /**
   * Key navigation starts from.
   *
   * @default The registry active key
   */
  readonly fromKey?: Key | null;

  /**
   * Whether navigation wraps at the first and last item.
   *
   * @default false
   */
  readonly loop?: boolean;
}

/** Options for locale-aware collection text lookup. */
export interface CollectionTextSearchOptions<Key extends CollectionKey> {
  /**
   * Search begins after this key so repeated input cycles matching items.
   *
   * @default The registry active key
   */
  readonly fromKey?: Key | null;

  /**
   * Match normalized prefixes or the complete normalized text.
   *
   * @default "prefix"
   */
  readonly match?: "prefix" | "exact";

  /**
   * Whether search wraps to the beginning of the collection.
   *
   * @default true
   */
  readonly loop?: boolean;

  /** Per-search locale comparator override. */
  readonly collator?: Intl.Collator;
}

/** Idempotent lifecycle handle returned by {@link CollectionRegistry.register}. */
export interface CollectionRegistration<Key extends CollectionKey> {
  /** Registered stable key. */
  readonly key: Key;

  /** Whether this exact registration still owns its key. */
  readonly registered: ComputedRef<boolean>;

  /** Remove this registration and report whether it was present. */
  readonly unregister: () => boolean;
}

/** Type-safe registry shared by compound collection components. */
export interface CollectionRegistry<Key extends CollectionKey, Value> {
  /** All items in deterministic virtual, DOM, or registration order. */
  readonly items: ComputedRef<readonly CollectionItem<Key, Value>[]>;

  /** Items eligible for focus navigation under the disabled policy. */
  readonly navigableItems: ComputedRef<readonly CollectionItem<Key, Value>[]>;

  /** Logical focus key used by roving-tabindex and `aria-activedescendant` adapters. */
  readonly activeKey: ComputedRef<Key | null>;

  /** Register one item. Vue scope disposal unregisters it automatically. */
  readonly register: (input: CollectionItemInput<Key, Value>) => CollectionRegistration<Key>;

  /** Resolve the latest immutable snapshot for a key. */
  readonly getItem: (key: Key) => CollectionItem<Key, Value> | undefined;

  /**
   * Re-resolve reactive sources, DOM order, and extracted text immediately.
   *
   * A scoped MutationObserver calls this automatically for connected items;
   * the explicit hook supports custom renderers and observer-free environments.
   */
  readonly refresh: () => void;

  /** Set a registered, navigable active key, or clear it with `null`. */
  readonly setActiveKey: (key: Key | null) => boolean;

  /** Resolve a directional target without changing active state. */
  readonly getNavigationKey: (
    direction: CollectionNavigationDirection,
    options?: CollectionNavigationOptions<Key>,
  ) => Key | null;

  /** Move active state directionally and return the resulting target. */
  readonly moveActive: (
    direction: CollectionNavigationDirection,
    options?: CollectionNavigationOptions<Key>,
  ) => Key | null;

  /** Find the next locale-aware text match without changing active state. */
  readonly findByTextValue: (
    query: string,
    options?: CollectionTextSearchOptions<Key>,
  ) => Key | null;

  /** Move active state to the next locale-aware text match. */
  readonly moveActiveByTextValue: (
    query: string,
    options?: CollectionTextSearchOptions<Key>,
  ) => Key | null;

  /** Stop observation, clear items, and reject future mutations. */
  readonly dispose: () => boolean;
}

interface CollectionRecord<Key extends CollectionKey, Value> {
  readonly input: Readonly<CollectionItemInput<Key, Value>>;
  readonly sequence: number;
}

interface ResolvedCollectionRecord<Key extends CollectionKey, Value> {
  readonly item: CollectionItem<Key, Value>;
  readonly sequence: number;
}

const documentPositionDisconnected = 0x01;
const documentPositionPreceding = 0x02;
const documentPositionFollowing = 0x04;
const ignoredTextTags = new Set(["NOSCRIPT", "SCRIPT", "STYLE", "TEMPLATE"]);

/**
 * Normalize authored or extracted typeahead text without applying a locale.
 *
 * Unicode is normalized to NFC and every whitespace run becomes one ASCII
 * space. Locale-sensitive equality remains the responsibility of a collator.
 */
export function normalizeCollectionTextValue(value: string): string {
  return value.normalize("NFC").trim().replace(/\s+/gu, " ");
}

/**
 * Extract a practical accessible text value from an item element.
 *
 * `aria-labelledby` and `aria-label` take precedence, hidden descendants are
 * excluded unless explicitly referenced, and image/input fallbacks are
 * included. Complex widgets should still provide `textValue` explicitly when
 * their typeahead label differs from their accessible name.
 */
export function extractCollectionTextValue(element: Element): string {
  return normalizeCollectionTextValue(readElementText(element, new Set(), false));
}

/**
 * Create an SSR-safe, mutation-aware registry for one compound collection.
 *
 * The registry never moves DOM focus itself. Consumers bind `activeKey` to
 * roving tabindex or `aria-activedescendant`; synchronous recovery guarantees
 * that removing or disabling the active item selects the next navigable item,
 * then the previous one, before the mutation can leave focus state orphaned.
 */
export function createCollectionRegistry<Key extends CollectionKey, Value>(
  options: CollectionRegistryOptions<Key> = {},
): CollectionRegistry<Key, Value> {
  const disabledBehavior = options.disabledBehavior ?? "skip";
  const collator =
    options.collator ?? new Intl.Collator(undefined, { sensitivity: "base", usage: "search" });
  const records = new Map<Key, CollectionRecord<Key, Value>>();
  const revision = shallowRef(0);
  const activeKeyState = shallowRef<Key | null>(null);
  const activeKey = computed(() => activeKeyState.value);
  let sequence = 0;
  let disposed = false;
  let mutationObservers: MutationObserver[] = [];

  const items = computed<readonly CollectionItem<Key, Value>[]>(() => {
    void revision.value;
    const resolved = [...records.values()].map(resolveCollectionRecord);
    sortCollectionRecords(resolved);
    return Object.freeze(resolved.map(({ item }) => item));
  });
  const navigableItems = computed<readonly CollectionItem<Key, Value>[]>(() => {
    const resolved = items.value.filter(
      (item) => disabledBehavior === "focusable" || !item.disabled,
    );
    return Object.freeze(resolved);
  });

  const updateActiveKey = (key: Key | null, reason: CollectionActiveChangeReason): boolean => {
    const previousKey = activeKeyState.value;
    if (previousKey === key) return false;
    activeKeyState.value = key;
    options.onActiveKeyChange?.(Object.freeze({ key, previousKey, reason }));
    return true;
  };

  const stopRecovery = watch(
    navigableItems,
    (nextItems, previousItems) => {
      const activeKey = activeKeyState.value;
      if (activeKey === null || hasCollectionKey(nextItems, activeKey)) return;

      const reason = hasCollectionKey(items.value, activeKey) ? "item-disabled" : "item-removed";
      updateActiveKey(recoverCollectionKey(activeKey, previousItems, nextItems), reason);
    },
    { flush: "sync" },
  );

  const stopObservation = watch(
    items,
    (nextItems) => {
      for (const observer of mutationObservers) observer.disconnect();
      mutationObservers = observeCollectionMutations(nextItems, () => {
        if (!disposed) revision.value += 1;
      });
    },
    { flush: "sync", immediate: true },
  );

  const assertMutable = () => {
    if (disposed) {
      throw new Error("VIZE_UI_COLLECTION_DISPOSED: a disposed registry cannot be mutated");
    }
  };

  const refresh = () => {
    assertMutable();
    revision.value += 1;
  };

  const getItem = (key: Key) => items.value.find((item) => item.key === key);

  const setActiveKey = (key: Key | null): boolean => {
    assertMutable();
    if (key === null) return updateActiveKey(null, "programmatic");

    const item = getItem(key);
    if (item === undefined) {
      throw new Error(`VIZE_UI_COLLECTION_KEY_MISSING: no item is registered for ${String(key)}`);
    }
    if (disabledBehavior === "skip" && item.disabled) {
      throw new Error(`VIZE_UI_COLLECTION_KEY_DISABLED: ${String(key)} is not navigable`);
    }
    return updateActiveKey(key, "programmatic");
  };

  const getNavigationKey = (
    direction: CollectionNavigationDirection,
    navigationOptions: CollectionNavigationOptions<Key> = {},
  ): Key | null => {
    const candidates = navigableItems.value;
    if (candidates.length === 0) return null;
    if (direction === "first") return candidates[0]?.key ?? null;
    if (direction === "last") return candidates.at(-1)?.key ?? null;

    const fromKey =
      navigationOptions.fromKey === undefined ? activeKeyState.value : navigationOptions.fromKey;
    if (fromKey === null) {
      return direction === "next" ? (candidates[0]?.key ?? null) : (candidates.at(-1)?.key ?? null);
    }

    const candidateIndex = candidates.findIndex((item) => item.key === fromKey);
    if (candidateIndex >= 0) {
      const targetIndex = candidateIndex + (direction === "next" ? 1 : -1);
      const target = candidates[targetIndex];
      if (target !== undefined) return target.key;
      if (navigationOptions.loop === true) {
        return direction === "next"
          ? (candidates[0]?.key ?? null)
          : (candidates.at(-1)?.key ?? null);
      }
      return fromKey;
    }

    const allItems = items.value;
    const itemIndex = allItems.findIndex((item) => item.key === fromKey);
    if (itemIndex >= 0) {
      const step = direction === "next" ? 1 : -1;
      for (let index = itemIndex + step; index >= 0 && index < allItems.length; index += step) {
        const candidate = allItems[index];
        if (candidate !== undefined && hasCollectionKey(candidates, candidate.key)) {
          return candidate.key;
        }
      }
      if (navigationOptions.loop !== true) return null;
    }

    return direction === "next" ? (candidates[0]?.key ?? null) : (candidates.at(-1)?.key ?? null);
  };

  const moveActive = (
    direction: CollectionNavigationDirection,
    navigationOptions: CollectionNavigationOptions<Key> = {},
  ): Key | null => {
    assertMutable();
    const target = getNavigationKey(direction, navigationOptions);
    if (target !== null) updateActiveKey(target, "navigation");
    return target;
  };

  const findByTextValue = (
    query: string,
    searchOptions: CollectionTextSearchOptions<Key> = {},
  ): Key | null => {
    const normalizedQuery = normalizeCollectionTextValue(query);
    if (normalizedQuery.length === 0) return null;

    const candidates = navigableItems.value;
    if (candidates.length === 0) return null;
    const fromKey =
      searchOptions.fromKey === undefined ? activeKeyState.value : searchOptions.fromKey;
    const fromIndex =
      fromKey === null ? -1 : candidates.findIndex((candidate) => candidate.key === fromKey);
    const after = candidates.slice(fromIndex + 1);
    const before = searchOptions.loop === false ? [] : candidates.slice(0, fromIndex + 1);
    const searchCandidates = [...after, ...before];
    const searchCollator = searchOptions.collator ?? collator;
    const match = searchOptions.match ?? "prefix";

    for (const candidate of searchCandidates) {
      if (candidate.textValue.length === 0) continue;
      const matches =
        match === "exact"
          ? searchCollator.compare(candidate.textValue, normalizedQuery) === 0
          : collectionTextStartsWith(candidate.textValue, normalizedQuery, searchCollator);
      if (matches) return candidate.key;
    }
    return null;
  };

  const moveActiveByTextValue = (
    query: string,
    searchOptions: CollectionTextSearchOptions<Key> = {},
  ): Key | null => {
    assertMutable();
    const target = findByTextValue(query, searchOptions);
    if (target !== null) updateActiveKey(target, "typeahead");
    return target;
  };

  const register = (input: CollectionItemInput<Key, Value>): CollectionRegistration<Key> => {
    assertMutable();
    validateCollectionKey(input.key);
    if (records.has(input.key)) {
      throw new Error(
        `VIZE_UI_COLLECTION_KEY_DUPLICATE: ${String(input.key)} is already registered`,
      );
    }

    const record: CollectionRecord<Key, Value> = {
      input: Object.freeze({ ...input }),
      sequence: sequence++,
    };
    records.set(input.key, record);
    try {
      revision.value += 1;
    } catch (error) {
      // Reactive source validation is part of registration. Roll back the map
      // before rethrowing so a rejected item never poisons the registry.
      records.delete(input.key);
      revision.value += 1;
      throw error;
    }

    const unregister = () => {
      if (records.get(input.key) !== record) return false;
      records.delete(input.key);
      revision.value += 1;
      return true;
    };
    const registration = Object.freeze({
      key: input.key,
      registered: computed(() => {
        void revision.value;
        return records.get(input.key) === record;
      }),
      unregister,
    });

    if (getCurrentScope() !== undefined) onScopeDispose(unregister);
    return registration;
  };

  const dispose = (): boolean => {
    if (disposed) return false;
    disposed = true;
    stopRecovery();
    stopObservation();
    for (const observer of mutationObservers) observer.disconnect();
    mutationObservers = [];
    records.clear();
    revision.value += 1;
    updateActiveKey(null, "registry-disposed");
    return true;
  };

  const registry = Object.freeze({
    items,
    navigableItems,
    activeKey,
    register,
    getItem,
    refresh,
    setActiveKey,
    getNavigationKey,
    moveActive,
    findByTextValue,
    moveActiveByTextValue,
    dispose,
  });

  if (getCurrentScope() !== undefined) onScopeDispose(dispose);
  return registry;
}

function resolveCollectionRecord<Key extends CollectionKey, Value>(
  record: CollectionRecord<Key, Value>,
): ResolvedCollectionRecord<Key, Value> {
  const { input } = record;
  const element = input.element === undefined ? null : (toValue(input.element) ?? null);
  const explicitText = input.textValue === undefined ? undefined : toValue(input.textValue);
  const order = input.order === undefined ? undefined : toValue(input.order);
  if (order !== undefined && !Number.isSafeInteger(order)) {
    throw new Error(
      `VIZE_UI_COLLECTION_ORDER_VALUE: ${String(input.key)} requires a safe integer order`,
    );
  }

  return {
    sequence: record.sequence,
    item: Object.freeze({
      key: input.key,
      value: input.value,
      element,
      textValue:
        explicitText === null || explicitText === undefined
          ? element === null
            ? ""
            : extractCollectionTextValue(element)
          : normalizeCollectionTextValue(explicitText),
      disabled: input.disabled === undefined ? false : Boolean(toValue(input.disabled)),
      order,
    }),
  };
}

function sortCollectionRecords<Key extends CollectionKey, Value>(
  records: ResolvedCollectionRecord<Key, Value>[],
): void {
  const explicitOrderCount = records.filter(({ item }) => item.order !== undefined).length;
  if (explicitOrderCount > 0 && explicitOrderCount !== records.length) {
    throw new Error(
      "VIZE_UI_COLLECTION_ORDER_PARTIAL: every item must provide order when explicit ordering is used",
    );
  }

  if (explicitOrderCount === records.length && records.length > 0) {
    const seenOrders = new Set<number>();
    for (const { item } of records) {
      const order = item.order;
      if (order === undefined) continue;
      if (seenOrders.has(order)) {
        throw new Error(
          `VIZE_UI_COLLECTION_ORDER_DUPLICATE: order ${String(order)} is registered more than once`,
        );
      }
      seenOrders.add(order);
    }
    records.sort((left, right) => (left.item.order ?? 0) - (right.item.order ?? 0));
    return;
  }

  const ownerDocument = records[0]?.item.element?.ownerDocument;
  const hasCompleteDomOrder =
    ownerDocument !== undefined &&
    records.every(
      ({ item }) =>
        item.element !== null &&
        item.element.isConnected &&
        item.element.ownerDocument === ownerDocument,
    );
  if (!hasCompleteDomOrder) {
    records.sort((left, right) => left.sequence - right.sequence);
    return;
  }

  records.sort((left, right) => {
    const leftElement = left.item.element;
    const rightElement = right.item.element;
    if (leftElement !== null && rightElement !== null && leftElement !== rightElement) {
      const position = leftElement.compareDocumentPosition(rightElement);
      if ((position & documentPositionDisconnected) === 0) {
        if ((position & documentPositionFollowing) !== 0) return -1;
        if ((position & documentPositionPreceding) !== 0) return 1;
      }
    }
    return left.sequence - right.sequence;
  });
}

function observeCollectionMutations<Key extends CollectionKey, Value>(
  items: readonly CollectionItem<Key, Value>[],
  onMutation: () => void,
): MutationObserver[] {
  const elementsByDocument = new Map<Document, Element[]>();
  for (const { element } of items) {
    if (element === null) continue;
    const elements = elementsByDocument.get(element.ownerDocument) ?? [];
    elements.push(element);
    elementsByDocument.set(element.ownerDocument, elements);
  }

  const observers: MutationObserver[] = [];
  for (const [ownerDocument, elements] of elementsByDocument) {
    const MutationObserverConstructor = ownerDocument.defaultView?.MutationObserver;
    if (MutationObserverConstructor === undefined) continue;

    const observer = new MutationObserverConstructor(onMutation);
    const commonAncestor = elements.every((element) => element.isConnected)
      ? findCollectionObservationRoot(elements)
      : null;
    const targets =
      commonAncestor === null
        ? new Set<Node>([
            ...elements.map((element) => element.getRootNode()),
            ownerDocument.documentElement,
          ])
        : new Set<Node>([commonAncestor]);
    for (const target of targets) {
      observer.observe(target, {
        attributeFilter: [
          "alt",
          "aria-hidden",
          "aria-label",
          "aria-labelledby",
          "hidden",
          "inert",
          "title",
          "type",
          "value",
        ],
        attributes: true,
        characterData: true,
        childList: true,
        subtree: true,
      });
    }
    observers.push(observer);
  }
  return observers;
}

function findCollectionObservationRoot(elements: readonly Element[]): Node | null {
  const firstElement = elements[0];
  if (firstElement === undefined) return null;

  let candidate: Node | null = firstElement.parentNode ?? firstElement;
  while (candidate !== null) {
    if (
      elements.every((element) => candidate === element || candidate?.contains(element) === true)
    ) {
      return candidate;
    }
    candidate = candidate.parentNode;
  }
  return null;
}

function validateCollectionKey(key: CollectionKey): void {
  if (typeof key === "string") {
    if (key.length === 0 || containsAsciiControl(key)) {
      throw new Error(
        "VIZE_UI_COLLECTION_KEY_VALUE: a string key must be non-empty and contain no ASCII controls",
      );
    }
    return;
  }
  if (!Number.isSafeInteger(key) || Object.is(key, -0)) {
    throw new Error(
      "VIZE_UI_COLLECTION_KEY_VALUE: a numeric key must be a safe integer other than negative zero",
    );
  }
}

function containsAsciiControl(value: string): boolean {
  for (let index = 0; index < value.length; index++) {
    const code = value.charCodeAt(index);
    if (code <= 0x1f || code === 0x7f) return true;
  }
  return false;
}

function hasCollectionKey<Key extends CollectionKey, Value>(
  items: readonly CollectionItem<Key, Value>[],
  key: Key,
): boolean {
  return items.some((item) => item.key === key);
}

function recoverCollectionKey<Key extends CollectionKey, Value>(
  removedKey: Key,
  previousItems: readonly CollectionItem<Key, Value>[],
  nextItems: readonly CollectionItem<Key, Value>[],
): Key | null {
  if (nextItems.length === 0) return null;
  const nextKeys = new Set(nextItems.map((item) => item.key));
  const previousIndex = previousItems.findIndex((item) => item.key === removedKey);
  if (previousIndex < 0) return nextItems[0]?.key ?? null;

  for (let index = previousIndex + 1; index < previousItems.length; index++) {
    const key = previousItems[index]?.key;
    if (key !== undefined && nextKeys.has(key)) return key;
  }
  for (let index = previousIndex - 1; index >= 0; index--) {
    const key = previousItems[index]?.key;
    if (key !== undefined && nextKeys.has(key)) return key;
  }
  return nextItems[0]?.key ?? null;
}

function readElementText(element: Element, visited: Set<Element>, referenced: boolean): string {
  if (visited.has(element)) return "";
  visited.add(element);
  if (!referenced && isCollectionTextHidden(element)) return "";

  const labelledBy = element.getAttribute("aria-labelledby");
  if (labelledBy !== null) {
    const labelledText = labelledBy
      .split(/\s+/u)
      .filter((id) => id.length > 0)
      .map((id) => element.ownerDocument.getElementById(id))
      .filter((label): label is HTMLElement => label !== null)
      .map((label) => readElementText(label, visited, true))
      .join(" ");
    if (normalizeCollectionTextValue(labelledText).length > 0) return labelledText;
  }

  const ariaLabel = element.getAttribute("aria-label");
  if (ariaLabel !== null && normalizeCollectionTextValue(ariaLabel).length > 0) return ariaLabel;

  const tagName = element.tagName.toUpperCase();
  if (tagName === "IMG" || tagName === "AREA") {
    const alternative = element.getAttribute("alt");
    if (alternative !== null) return alternative;
  }
  if (tagName === "INPUT") {
    const type = element.getAttribute("type")?.toLowerCase();
    if (type === "button" || type === "reset" || type === "submit") {
      const value = (element as HTMLInputElement).value;
      if (value.length > 0) return value;
    }
  }

  if (!ignoredTextTags.has(tagName)) {
    const fragments: string[] = [];
    for (const child of element.childNodes) {
      if (child.nodeType === 3) {
        fragments.push(child.nodeValue ?? "");
      } else if (child.nodeType === 1) {
        fragments.push(readElementText(child as Element, visited, referenced));
      }
    }
    const content = fragments.join(" ");
    if (normalizeCollectionTextValue(content).length > 0) return content;
  }

  return element.getAttribute("title") ?? "";
}

function isCollectionTextHidden(element: Element): boolean {
  return (
    element.hasAttribute("hidden") ||
    element.hasAttribute("inert") ||
    element.getAttribute("aria-hidden")?.trim().toLowerCase() === "true"
  );
}

function collectionTextStartsWith(
  candidate: string,
  query: string,
  collator: Intl.Collator,
): boolean {
  let codeUnitLength = 0;
  for (const character of candidate) {
    codeUnitLength += character.length;
    if (collator.compare(candidate.slice(0, codeUnitLength), query) === 0) return true;
  }
  return false;
}
