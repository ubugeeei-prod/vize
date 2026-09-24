import { getCurrentScope, onMounted, shallowRef, watch } from "vue";

import { isValidSplitterLayout } from "./splitter-layout.ts";
import type {
  SplitterLayout,
  SplitterPersistedLayout,
  SplitterPersistenceOptions,
  SplitterStorage,
} from "./splitter-types.ts";

const setupDiagnostic = "VIZE_UI_SPLITTER_PERSISTENCE_SETUP";

function resolveStorage(options: SplitterPersistenceOptions): SplitterStorage | null {
  const configured = options.storage;
  if (typeof configured === "function") return configured() ?? null;
  if (configured !== undefined) return configured;
  try {
    return typeof globalThis.localStorage === "undefined" ? null : globalThis.localStorage;
  } catch {
    // Access to localStorage throws when storage is blocked by privacy settings.
    return null;
  }
}

/** Parse a stored layout, returning `undefined` for missing or malformed values. */
export function parseSplitterLayout(value: string | null): SplitterLayout | undefined {
  if (value === null) return undefined;
  try {
    const parsed: unknown = JSON.parse(value);
    return Array.isArray(parsed) && isValidSplitterLayout(parsed, parsed.length)
      ? Object.freeze(parsed.map((size) => Number(size)))
      : undefined;
  } catch {
    return undefined;
  }
}

/**
 * Persist a SplitterGroup layout in synchronous storage.
 *
 * Bind the returned ref with `v-model:layout`. It stays `undefined`, keeping
 * the group on its SSR-stable defaults, until the stored layout is read after
 * mount; with `immediate: true` it is read during setup instead, which is only
 * hydration-safe for storage the server can read too, such as a cookie adapter.
 * Every later layout change is written back. Stored layouts with the wrong
 * panel count are ignored by the group and replaced on the next resize.
 *
 * @example
 * ```ts
 * const layout = useSplitterPersistence({ key: "editor-layout" });
 * ```
 */
export function useSplitterPersistence(
  options: SplitterPersistenceOptions,
): SplitterPersistedLayout {
  if (getCurrentScope() === undefined) {
    throw new Error(`${setupDiagnostic}: call inside component setup or an active effect scope`);
  }
  const layout = shallowRef<SplitterLayout | undefined>(undefined);
  let storage: SplitterStorage | null = null;
  let hydrated = false;

  const load = () => {
    storage = resolveStorage(options);
    try {
      layout.value = parseSplitterLayout(storage?.getItem(options.key) ?? null) ?? layout.value;
    } catch {
      layout.value = undefined;
    }
    hydrated = true;
  };

  watch(layout, (next) => {
    if (!hydrated || next === undefined) return;
    try {
      storage?.setItem(options.key, JSON.stringify(next));
    } catch {
      // Quota and privacy errors leave the in-memory layout authoritative.
    }
  });

  if (options.immediate === true) load();
  else onMounted(load);
  return layout;
}
