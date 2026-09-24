import { inject } from "vue";
import type { ComputedRef, InjectionKey, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { NotificationCenterState, NotificationStore } from "./notification-center-types.ts";

/**
 * Injection key under which NotificationCenterRoot publishes its store. The
 * payload type is the root's `Data`; `useNotificationCenter<Data>()` reads it back.
 */
export const notificationStoreKey: InjectionKey<NotificationStore<unknown>> =
  Symbol("NotificationStore");

/** Root state shared with the trigger, list, items, and empty state. */
export interface NotificationCenterContextValue {
  readonly store: NotificationStore<unknown>;
  readonly listId: ComputedRef<string>;
  readonly triggerId: ComputedRef<string>;
  readonly label: ComputedRef<string>;
  readonly inline: ComputedRef<boolean>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<NotificationCenterState>;
  readonly triggerElement: ShallowRef<HTMLButtonElement | null>;
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;
  readonly itemId: (id: string) => string;
}

export const notificationCenterContext =
  createContext<NotificationCenterContextValue>("NotificationCenter");

/**
 * Read the store of the nearest NotificationCenterRoot.
 *
 * `Data` must match the root's `Data` (or its `store` prop); like Vue's
 * `inject`, the payload type is a caller-side contract.
 *
 * @throws {Error} `VIZE_UI_CONTEXT_MISSING` outside a NotificationCenterRoot.
 */
export function useNotificationCenter<Data = unknown>(): NotificationStore<Data> {
  const key: InjectionKey<NotificationStore<Data>> = notificationStoreKey;
  const store = inject(key, null);
  if (store === null) {
    throw new Error(
      "VIZE_UI_CONTEXT_MISSING: useNotificationCenter requires a matching NotificationCenterRoot",
    );
  }
  return store;
}
