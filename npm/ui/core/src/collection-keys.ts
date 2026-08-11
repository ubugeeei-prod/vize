import type { CollectionItem, CollectionKey } from "./collection-types.ts";

/** Reject keys that cannot serve as stable, serializable item identities. */
export function validateCollectionKey(key: CollectionKey): void {
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

/** Whether a resolved item list still contains a key. */
export function hasCollectionKey<Key extends CollectionKey, Value>(
  items: readonly CollectionItem<Key, Value>[],
  key: Key,
): boolean {
  return items.some((item) => item.key === key);
}

/**
 * Pick the logical focus successor for a key that left the navigable set.
 *
 * The next navigable item wins, then the previous one, so removing or
 * disabling the active item never orphans logical focus.
 */
export function recoverCollectionKey<Key extends CollectionKey, Value>(
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

function containsAsciiControl(value: string): boolean {
  for (let index = 0; index < value.length; index++) {
    const code = value.charCodeAt(index);
    if (code <= 0x1f || code === 0x7f) return true;
  }
  return false;
}
