import { computed, getCurrentScope, onScopeDispose, shallowRef, watch } from "vue";

import {
  hasCollectionKey,
  recoverCollectionKey,
  validateCollectionKey,
} from "./collection-keys.ts";
import { observeCollectionMutations } from "./collection-observer.ts";
import { resolveCollectionRecord, sortCollectionRecords } from "./collection-order.ts";
import type { CollectionRecord } from "./collection-order.ts";
import { collectionTextStartsWith, normalizeCollectionTextValue } from "./collection-text.ts";
import type {
  CollectionActiveChangeReason,
  CollectionItem,
  CollectionItemInput,
  CollectionKey,
  CollectionNavigationDirection,
  CollectionNavigationOptions,
  CollectionRegistration,
  CollectionRegistry,
  CollectionRegistryOptions,
  CollectionTextSearchOptions,
} from "./collection-types.ts";

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
  let observedElements: readonly (Element | null)[] | undefined;
  let observedConnected: readonly boolean[] = [];

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
      const nextElements = nextItems.map(({ element }) => element);
      const nextConnected = nextElements.map((element) => element?.isConnected === true);
      if (
        observedElements !== undefined &&
        observedElements.length === nextElements.length &&
        observedElements.every(
          (element, index) =>
            element === nextElements[index] && observedConnected[index] === nextConnected[index],
        )
      ) {
        return;
      }
      for (const observer of mutationObservers) observer.disconnect();
      mutationObservers = observeCollectionMutations(nextItems, () => {
        if (!disposed) revision.value += 1;
      });
      observedElements = nextElements;
      observedConnected = nextConnected;
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
