import { computed, readonly, ref, shallowReadonly, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Lifecycle states of a service worker. */
export type ServiceWorkerStateName =
  | "parsed"
  | "installing"
  | "installed"
  | "activating"
  | "activated"
  | "redundant";

/** Minimal `ServiceWorker` consumed by {@link useServiceWorker}. */
export interface ServiceWorkerLike extends EventTarget {
  /** Current lifecycle state; changes fire `statechange`. */
  readonly state: ServiceWorkerStateName;

  /** Post a message to the worker. */
  postMessage(message: unknown): void;
}

/** Minimal `ServiceWorkerRegistration` consumed by {@link useServiceWorker}. */
export interface ServiceWorkerRegistrationLike extends EventTarget {
  /** Worker being installed, if any; a new one fires `updatefound`. */
  readonly installing: ServiceWorkerLike | null;

  /** Installed worker waiting to take over, if any. */
  readonly waiting: ServiceWorkerLike | null;

  /** Active worker, if any. */
  readonly active: ServiceWorkerLike | null;

  /** Registration scope URL. */
  readonly scope: string;

  /** Check the server for an updated script. */
  update(): Promise<unknown>;

  /** Unregister; resolves whether the registration was removed. */
  unregister(): Promise<boolean>;
}

/** Options forwarded to `ServiceWorkerContainer.register`. */
export interface ServiceWorkerRegisterOptions {
  /**
   * Registration scope.
   *
   * @default the script's directory
   */
  readonly scope?: string;

  /**
   * Script type.
   *
   * @default "classic"
   */
  readonly type?: "classic" | "module";

  /**
   * HTTP cache policy for update checks.
   *
   * @default "imports"
   */
  readonly updateViaCache?: "imports" | "all" | "none";
}

/** Minimal `ServiceWorkerContainer` (`navigator.serviceWorker`). */
export interface ServiceWorkerContainerLike extends EventTarget {
  /** Worker controlling this page; changes fire `controllerchange`. */
  readonly controller: ServiceWorkerLike | null;

  /** Register a service worker script. */
  register(
    scriptUrl: string | URL,
    options?: ServiceWorkerRegisterOptions,
  ): Promise<ServiceWorkerRegistrationLike>;
}

/** States of the registration's workers (`null` when absent). */
export interface ServiceWorkerStates {
  /** State of the installing worker. */
  readonly installing: ServiceWorkerStateName | null;

  /** State of the waiting worker. */
  readonly waiting: ServiceWorkerStateName | null;

  /** State of the active worker. */
  readonly active: ServiceWorkerStateName | null;
}

/** Options for {@link useServiceWorker}. */
export interface UseServiceWorkerOptions {
  /**
   * Service worker container for alternate runtimes and tests.
   *
   * @default window.navigator.serviceWorker when it exists
   */
  readonly container?: MaybeRefOrGetter<ServiceWorkerContainerLike | null | undefined>;

  /**
   * Registration scope.
   *
   * @default undefined (the script's directory)
   */
  readonly scope?: string;

  /**
   * Script type.
   *
   * @default "classic"
   */
  readonly type?: "classic" | "module";

  /**
   * HTTP cache policy for update checks.
   *
   * @default "imports"
   */
  readonly updateViaCache?: "imports" | "all" | "none";

  /**
   * Register as soon as the composable is created (browser only).
   *
   * @default true
   */
  readonly immediate?: boolean;

  /**
   * Message posted to the waiting worker by `skipWaiting()`.
   *
   * @default { type: "SKIP_WAITING" }
   */
  readonly skipWaitingMessage?: unknown;

  /**
   * Called on `controllerchange` (for example to reload the page after a
   * new worker took control).
   *
   * @default undefined
   */
  readonly onControllerChange?: (controller: ServiceWorkerLike | null) => void;
}

/** Reactive state and actions returned by {@link useServiceWorker}. */
export interface ServiceWorkerControls {
  /** Whether the Service Worker API is available. */
  readonly supported: ComputedRef<boolean>;

  /** Current registration, or `null` before registering. */
  readonly registration: Readonly<ShallowRef<ServiceWorkerRegistrationLike | null>>;

  /** States of the installing, waiting and active workers. */
  readonly state: Readonly<Ref<ServiceWorkerStates>>;

  /** Whether a waiting worker is ready to replace the controlling one. */
  readonly updateAvailable: ComputedRef<boolean>;

  /** Most recent failure, cleared by the next successful action. */
  readonly error: Readonly<ShallowRef<unknown>>;

  /**
   * Register (or re-register) the script.
   *
   * @returns The registration, or `null` when unsupported or on failure.
   */
  readonly register: () => Promise<ServiceWorkerRegistrationLike | null>;

  /**
   * Check the server for an updated script.
   *
   * @returns Whether the check ran without failing.
   */
  readonly update: () => Promise<boolean>;

  /**
   * Post the skip-waiting message to the waiting worker.
   *
   * @returns Whether a waiting worker received the message.
   */
  readonly skipWaiting: () => boolean;

  /**
   * Unregister the service worker.
   *
   * @returns Whether the registration was removed.
   */
  readonly unregister: () => Promise<boolean>;
}

const emptyStates: ServiceWorkerStates = { installing: null, waiting: null, active: null };

function browserContainer(): ServiceWorkerContainerLike | undefined {
  if (typeof window === "undefined") return undefined;
  const { navigator } = window;
  return "serviceWorker" in navigator && navigator.serviceWorker
    ? navigator.serviceWorker
    : undefined;
}

/**
 * Register a service worker and follow its update lifecycle.
 *
 * Tracks `updatefound` and each worker's `statechange` so `state` mirrors
 * the installing, waiting and active workers. `updateAvailable` turns true
 * once an installed worker waits while another one controls the page;
 * `skipWaiting()` then posts the configured message to it, and
 * `onControllerChange` runs once it takes control. Listeners are removed
 * when the owning reactive scope stops; outside a scope they live as long as
 * the registration.
 *
 * Server rendering: nothing is registered, `supported` is false and every
 * state is `null`.
 *
 * @example
 * ```ts
 * const sw = useServiceWorker("/sw.js", {
 *   onControllerChange: () => window.location.reload(),
 * });
 * const applyUpdate = () => sw.skipWaiting();
 * ```
 *
 * @param scriptUrl Service worker script URL.
 * @param options Container override, registration options and update hooks.
 * @default options {}
 * @returns Registration state and actions.
 */
export function useServiceWorker(
  scriptUrl: MaybeRefOrGetter<string | URL>,
  options: UseServiceWorkerOptions = {},
): ServiceWorkerControls {
  const registration = shallowRef<ServiceWorkerRegistrationLike | null>(null);
  const state = ref<ServiceWorkerStates>(emptyStates);
  const controller = shallowRef<ServiceWorkerLike | null>(null);
  const error = shallowRef<unknown>(undefined);
  const watched = new Set<ServiceWorkerLike>();
  let active = true;

  const resolveContainer = (): ServiceWorkerContainerLike | undefined =>
    options.container === undefined
      ? browserContainer()
      : (toValue(options.container) ?? undefined);

  const sync = (): void => {
    const current = registration.value;
    if (!current) {
      state.value = emptyStates;
      return;
    }
    for (const worker of [current.installing, current.waiting, current.active]) {
      if (worker && !watched.has(worker)) {
        watched.add(worker);
        worker.addEventListener("statechange", sync);
      }
    }
    state.value = {
      installing: current.installing?.state ?? null,
      waiting: current.waiting?.state ?? null,
      active: current.active?.state ?? null,
    };
  };

  const unwatchWorkers = (): void => {
    for (const worker of watched) worker.removeEventListener("statechange", sync);
    watched.clear();
  };

  watch(
    registration,
    (current, _previous, onCleanup) => {
      sync();
      if (!current) return;
      current.addEventListener("updatefound", sync);
      onCleanup(() => {
        current.removeEventListener("updatefound", sync);
        unwatchWorkers();
      });
    },
    { flush: "sync" },
  );

  watch(
    resolveContainer,
    (container, _previous, onCleanup) => {
      controller.value = container?.controller ?? null;
      if (!container) return;
      const onChange = (): void => {
        controller.value = container.controller;
        sync();
        options.onControllerChange?.(container.controller);
      };
      container.addEventListener("controllerchange", onChange);
      onCleanup(() => container.removeEventListener("controllerchange", onChange));
    },
    { immediate: true, flush: "sync" },
  );

  const register = async (): Promise<ServiceWorkerRegistrationLike | null> => {
    const container = resolveContainer();
    if (!container) return null;
    const registerOptions: {
      scope?: string;
      type?: "classic" | "module";
      updateViaCache?: "imports" | "all" | "none";
    } = {};
    if (options.scope !== undefined) registerOptions.scope = options.scope;
    if (options.type !== undefined) registerOptions.type = options.type;
    if (options.updateViaCache !== undefined) {
      registerOptions.updateViaCache = options.updateViaCache;
    }
    try {
      const next = await container.register(toValue(scriptUrl), registerOptions);
      if (!active) return next;
      registration.value = next;
      error.value = undefined;
      return next;
    } catch (cause) {
      if (active) error.value = cause;
      return null;
    }
  };

  const update = async (): Promise<boolean> => {
    const current = registration.value;
    if (!current) return false;
    try {
      await current.update();
      error.value = undefined;
      sync();
      return true;
    } catch (cause) {
      error.value = cause;
      return false;
    }
  };

  const skipWaiting = (): boolean => {
    const waiting = registration.value?.waiting;
    if (!waiting) return false;
    waiting.postMessage(
      "skipWaitingMessage" in options ? options.skipWaitingMessage : { type: "SKIP_WAITING" },
    );
    return true;
  };

  const unregister = async (): Promise<boolean> => {
    const current = registration.value;
    if (!current) return false;
    try {
      const removed = await current.unregister();
      if (removed) registration.value = null;
      error.value = undefined;
      return removed;
    } catch (cause) {
      error.value = cause;
      return false;
    }
  };

  if ((options.immediate ?? true) && resolveContainer()) void register();

  tryOnScopeDispose(() => {
    active = false;
    registration.value = null;
  });

  return {
    supported: computed(() => resolveContainer() !== undefined),
    registration: shallowReadonly(registration),
    state: readonly(state),
    updateAvailable: computed(() => state.value.waiting !== null && controller.value !== null),
    error: readonly(error),
    register,
    update,
    skipWaiting,
    unregister,
  };
}
