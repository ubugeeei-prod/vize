import type { ComputedRef, InjectionKey } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  ToastDismissReason,
  ToastRecord,
  ToastStore,
  ToastSwipeDirection,
  ToastSwipeState,
} from "./toast-types.ts";

/**
 * Injection key under which ToastProvider publishes its store. The payload
 * type is the provider's `Data`; `useToast<Data>()` reads it back.
 */
export const toastStoreKey: InjectionKey<ToastStore<unknown>> = Symbol("ToastStore");

/** Provider configuration shared with viewports and toasts. */
export interface ToastProviderContextValue {
  readonly label: ComputedRef<string>;
  readonly hotkey: ComputedRef<readonly string[]>;
  readonly swipeDirection: ComputedRef<ToastSwipeDirection>;
  readonly swipeThreshold: ComputedRef<number>;
}

export const toastProviderContext = createContext<ToastProviderContextValue>("ToastProvider");

/** Viewport services shared with rendered toasts. */
export interface ToastViewportContextValue {
  readonly focus: (options?: FocusOptions) => void;
}

export const toastViewportContext = createContext<ToastViewportContextValue>("ToastViewport");

/** Per-toast state shared with title, description, action, and close parts. */
export interface ToastRootContextValue {
  readonly toast: ComputedRef<ToastRecord<unknown>>;
  readonly titleId: ComputedRef<string>;
  readonly descriptionId: ComputedRef<string>;
  readonly swipe: ComputedRef<ToastSwipeState | null>;
  readonly dismiss: (reason?: ToastDismissReason) => boolean;
}

export const toastRootContext = createContext<ToastRootContextValue>("ToastRoot");
