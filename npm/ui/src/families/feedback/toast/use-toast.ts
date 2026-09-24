import { inject } from "vue";
import type { InjectionKey } from "vue";

import { toastStoreKey } from "./toast-context.ts";
import type { ToastStore } from "./toast-types.ts";

/**
 * Read the store of the nearest ToastProvider.
 *
 * `Data` must match the provider's `Data` (or `store` prop); like Vue's
 * `inject`, the payload type is a caller-side contract.
 *
 * @throws {Error} `VIZE_UI_CONTEXT_MISSING` outside a ToastProvider.
 */
export function useToast<Data = unknown>(): ToastStore<Data> {
  const key: InjectionKey<ToastStore<Data>> = toastStoreKey;
  const store = inject(key, null);
  if (store === null) {
    throw new Error("VIZE_UI_CONTEXT_MISSING: useToast requires a matching ToastProvider");
  }
  return store;
}
