/**
 * Recent-history model for the Autocomplete preset.
 *
 * Pure, SSR-safe helpers: the newest entry comes first, duplicates (under the
 * configured equality) move to the front instead of repeating, and the list
 * is capped. Persistence is an injectable adapter so no storage API is ever
 * touched during setup or on the server.
 */

/** Persistence adapter for recent history, e.g. backed by `localStorage`. */
export interface AutocompleteHistoryStorage<T> {
  /** Read persisted entries, or `null` when nothing is stored. */
  readonly read: () => readonly T[] | null;

  /** Persist the current entries. */
  readonly write: (entries: readonly T[]) => void;
}

/** Add `entry` to the front of `history`, de-duplicating and capping to `max`. */
export function pushAutocompleteHistory<T>(
  history: readonly T[],
  entry: T,
  max: number,
  equals: (left: T, right: T) => boolean,
): readonly T[] {
  const limit = Number.isFinite(max) ? Math.max(0, Math.floor(max)) : history.length + 1;
  const rest = history.filter((candidate) => !equals(candidate, entry));
  return Object.freeze([entry, ...rest].slice(0, limit));
}

/** Remove `entry` from `history`. */
export function removeAutocompleteHistory<T>(
  history: readonly T[],
  entry: T,
  equals: (left: T, right: T) => boolean,
): readonly T[] {
  return Object.freeze(history.filter((candidate) => !equals(candidate, entry)));
}

/** Options for {@link createWebStorageHistory}. */
export interface WebStorageHistoryOptions<T> {
  /** Storage key. */
  readonly key: string;

  /** Convert entries to JSON-safe data. @default identity */
  readonly serialize?: (entries: readonly T[]) => unknown;

  /** Validate and convert stored data back into entries; return `null` to ignore it. */
  readonly parse: (data: unknown) => readonly T[] | null;

  /** Storage to use; resolved lazily so SSR never touches `window`. @default globalThis.localStorage */
  readonly storage?: () => Pick<Storage, "getItem" | "setItem"> | null;
}

function defaultStorage(): Pick<Storage, "getItem" | "setItem"> | null {
  try {
    return typeof globalThis.localStorage === "undefined" ? null : globalThis.localStorage;
  } catch {
    return null;
  }
}

/**
 * Web Storage adapter. Read and write failures (quota, privacy mode, corrupt
 * JSON) are swallowed so history never breaks input handling.
 */
export function createWebStorageHistory<T>(
  options: WebStorageHistoryOptions<T>,
): AutocompleteHistoryStorage<T> {
  const storage = options.storage ?? defaultStorage;
  return Object.freeze({
    read: () => {
      try {
        const raw = storage()?.getItem(options.key);
        return raw === null || raw === undefined ? null : options.parse(JSON.parse(raw));
      } catch {
        return null;
      }
    },
    write: (entries: readonly T[]) => {
      try {
        const data = options.serialize === undefined ? entries : options.serialize(entries);
        storage()?.setItem(options.key, JSON.stringify(data));
      } catch {
        // Persistence is best effort.
      }
    },
  });
}
