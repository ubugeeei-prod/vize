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
 * async rejections reject the returned promise. SSR-safe, with one caveat:
 * server component scopes are not disposed after rendering, so register
 * listeners during server `setup` only on request-local hooks, never on
 * module-level ones.
 *
 * @example
 * ```ts
 * const saved = createEventHook<[id: string]>();
 * saved.on((id) => toast(`Saved ${id}`));
 * await saved.trigger("doc-1");
 * ```
 *
 * @returns The event hook.
 */
export function createEventHook<Arguments extends readonly unknown[] = []>(): EventHook<Arguments> {
  const listeners = new Set<EventHookListener<Arguments>>();

  const off = (listener: EventHookListener<Arguments>): void => {
    listeners.delete(listener);
  };

  const on = (listener: EventHookListener<Arguments>): EventHookSubscription => {
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
