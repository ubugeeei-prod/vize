import { getCurrentInstance } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Listener registered on an {@link EventHook}. */
export type EventHookListener<Arguments extends readonly unknown[]> = (
  ...args: Arguments
) => unknown;

/** Registration handle returned by {@link EventHook.on}. */
export interface EventHookSubscription {
  /** Remove the listener. Idempotent. */
  readonly off: () => void;
}

/** Options for {@link createEventHook}. */
export interface CreateEventHookOptions {
  /**
   * What `on()` does when a hook created outside any component (a
   * module-level hook shared by every request) is subscribed to from a
   * component `setup` during server rendering. Vue never disposes server
   * component scopes, so such listeners would accumulate across requests
   * and keep every rendered component alive.
   *
   * - `"ignore"`: skip the registration (the returned `off` is a no-op) and
   *   warn once in development.
   * - `"register"`: register anyway; the caller must call `off()`.
   *
   * Hooks created inside a component (request-local hooks) are unaffected.
   *
   * @default "ignore"
   */
  readonly serverListeners?: "ignore" | "register";
}

/** Typed event emitter created by {@link createEventHook}. */
export interface EventHook<Arguments extends readonly unknown[] = []> {
  /**
   * Add a listener. Inside an effect scope it is removed automatically when
   * the scope stops. Adding the same function twice registers it once.
   */
  readonly on: (listener: EventHookListener<Arguments>) => EventHookSubscription;

  /** Remove a listener. */
  readonly off: (listener: EventHookListener<Arguments>) => void;

  /**
   * Call every listener with `args`, in registration order.
   *
   * @returns Settles with every listener's (awaited) result.
   */
  readonly trigger: (...args: Arguments) => Promise<unknown[]>;

  /** Remove every listener. */
  readonly clear: () => void;

  /** Number of registered listeners. */
  readonly size: () => number;
}

/**
 * Create a small, strongly typed event emitter.
 *
 * The payload is a tuple type, so events can carry several named arguments
 * (`createEventHook<[id: string, value: number]>()`) and `trigger` checks
 * them. Listeners registered inside an effect scope unsubscribe themselves
 * when it stops. Synchronous listener errors propagate from `trigger`;
 * async rejections reject the returned promise.
 *
 * SSR: Vue does not dispose component scopes after a server render, so a
 * listener that a server `setup` adds to a module-level hook would never be
 * removed and would pile up once per request. By default such registrations
 * are therefore skipped (with a one-time development warning); see
 * {@link CreateEventHookOptions.serverListeners}. Hooks created inside a
 * component, and every registration in the browser, behave normally.
 *
 * @example
 * ```ts
 * const saved = createEventHook<[id: string]>();
 * saved.on((id) => toast(`Saved ${id}`));
 * await saved.trigger("doc-1");
 * ```
 *
 * @param options Server-side registration policy.
 * @default options {}
 * @returns The event hook.
 */
export function createEventHook<Arguments extends readonly unknown[] = []>(
  options: CreateEventHookOptions = {},
): EventHook<Arguments> {
  const listeners = new Set<EventHookListener<Arguments>>();
  // A hook created outside any component is shared by every server request.
  const shared = getCurrentInstance() === null;
  let warned = false;

  const off = (listener: EventHookListener<Arguments>): void => {
    listeners.delete(listener);
  };

  const on = (listener: EventHookListener<Arguments>): EventHookSubscription => {
    if (
      shared &&
      (options.serverListeners ?? "ignore") === "ignore" &&
      typeof window === "undefined" &&
      getCurrentInstance() !== null
    ) {
      if (!warned && isDevelopment()) {
        warned = true;
        console.warn(
          '[VIZE_COMPOSE_EVENT_HOOK_SERVER_LISTENER] a module-level event hook was subscribed to during server rendering; the listener was skipped because server component scopes are never disposed. Create the hook inside the component tree, or pass { serverListeners: "register" } and call off() yourself.',
        );
      }
      return { off: () => undefined };
    }
    listeners.add(listener);
    const subscription = { off: () => off(listener) };
    tryOnScopeDispose(subscription.off);
    return subscription;
  };

  const trigger = (...args: Arguments): Promise<unknown[]> =>
    Promise.all([...listeners].map((listener) => listener(...args)));

  return {
    on,
    off,
    trigger,
    clear: () => {
      listeners.clear();
    },
    size: () => listeners.size,
  };
}

function isDevelopment(): boolean {
  return typeof process === "undefined" || process.env["NODE_ENV"] !== "production";
}
