import { getCurrentScope, onMounted, shallowRef, watch } from "vue";

import { parseWindowLayout } from "./window-manager-model.ts";
import type { WindowLayout } from "./window-manager-model.ts";
import type {
  WindowLayoutPersistenceOptions,
  WindowLayoutStorage,
  WindowPersistedLayout,
} from "./window-manager-types.ts";

const setupDiagnostic = "VIZE_UI_WINDOW_MANAGER_PERSISTENCE_SETUP";

function resolveStorage(options: WindowLayoutPersistenceOptions): WindowLayoutStorage | null {
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

/**
 * Persist a WindowManager layout (positions, sizes, modes, stacking) in
 * synchronous storage.
 *
 * Bind the returned ref with `v-model:layout`. It stays `undefined` (the
 * SSR-stable defaults) until the stored layout is read after mount; with
 * `immediate: true` it is read during setup instead, which is only
 * hydration-safe for storage the server can read too. Later changes are
 * written back; malformed stored values are ignored.
 *
 * @param options Storage key, adapter, and timing.
 * @returns Ref for `v-model:layout`.
 */
export function useWindowLayoutPersistence(
  options: WindowLayoutPersistenceOptions,
): WindowPersistedLayout {
  if (getCurrentScope() === undefined) {
    throw new Error(`${setupDiagnostic}: call inside component setup or an active effect scope`);
  }
  const layout = shallowRef<WindowLayout | undefined>(undefined);
  let storage: WindowLayoutStorage | null = null;
  let hydrated = false;

  const load = (): void => {
    storage = resolveStorage(options);
    try {
      layout.value = parseWindowLayout(storage?.getItem(options.key) ?? null) ?? layout.value;
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
