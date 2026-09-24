import { readonly, ref, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

/** Window-like capability observed by {@link useIdle}. */
export interface IdleHost extends EventTarget {
  /** Document whose visibility changes count as activity when it becomes visible. */
  readonly document?: EventTarget & { readonly hidden?: boolean };
}

/** Default activity events. */
export const DEFAULT_IDLE_EVENTS = [
  "mousemove",
  "mousedown",
  "resize",
  "keydown",
  "touchstart",
  "wheel",
] as const;

/** Options for {@link useIdle}. */
export interface UseIdleOptions {
  /**
   * Window events that count as activity.
   *
   * @default DEFAULT_IDLE_EVENTS
   */
  readonly events?: readonly string[];

  /**
   * Treat the document becoming visible again as activity.
   *
   * @default true
   */
  readonly listenForVisibilityChange?: boolean;

  /**
   * Idle state exposed during server rendering and before the timer starts.
   *
   * @default false
   */
  readonly initialState?: boolean;

  /**
   * Clock used for `lastActive`.
   *
   * @default Date.now
   */
  readonly now?: () => number;

  /**
   * Owns the idle timer.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TimeoutScheduler;

  /**
   * Reactive window capability for alternate runtimes and tests.
   *
   * @default globalThis.window when available
   */
  readonly host?: MaybeRefOrGetter<IdleHost | null | undefined>;
}

/** Reactive idle state returned by {@link useIdle}. */
export interface IdleControls {
  /** Whether no activity happened within the timeout. */
  readonly idle: Readonly<Ref<boolean>>;
  /** Epoch milliseconds of the latest activity; `null` until tracking starts on the client. */
  readonly lastActive: Readonly<Ref<number | null>>;
  /** Record activity now and restart the timer. No-op until tracking starts on the client. */
  readonly reset: () => void;
}

const defaultScheduler: TimeoutScheduler = {
  setTimeout: (callback, delayMs) => globalThis.setTimeout(callback, delayMs),
  clearTimeout: (handle) => {
    globalThis.clearTimeout(handle as ReturnType<typeof setTimeout>);
  },
};

/**
 * Detect user inactivity.
 *
 * Any configured activity event resets a single timer; when it fires, `idle`
 * becomes `true` until the next activity. No timer runs during server
 * rendering (`idle` is `initialState`, `lastActive` is `null`), so the
 * output is hydration-stable. Listeners and the timer are released with the
 * owning reactive scope.
 *
 * @param timeout Inactivity window in milliseconds.
 * @param options Activity events, clock, scheduler, and window capability.
 * @default timeout 60000
 * @default options {}
 * @returns Reactive idle flag, last activity time, and a reset action.
 */
export function useIdle(timeout = 60_000, options: UseIdleOptions = {}): IdleControls {
  const {
    events = DEFAULT_IDLE_EVENTS,
    listenForVisibilityChange = true,
    initialState = false,
    now = Date.now,
    scheduler = defaultScheduler,
  } = options;
  const idle = ref(initialState);
  const lastActive = ref<number | null>(null);
  let handle: unknown;
  let hasTimer = false;
  let tracking = false;

  const clear = (): void => {
    if (!hasTimer) return;
    scheduler.clearTimeout(handle);
    hasTimer = false;
  };
  const reset = (): void => {
    if (!tracking) return;
    idle.value = false;
    lastActive.value = now();
    clear();
    handle = scheduler.setTimeout(() => {
      hasTimer = false;
      idle.value = true;
    }, timeout);
    hasTimer = true;
  };

  const stop = watch(
    () => (options.host === undefined ? browserIdleHost() : toValue(options.host)),
    (host, _previous, onCleanup) => {
      if (!host) return;
      tracking = true;
      const listenerOptions: AddEventListenerOptions = { passive: true };
      for (const event of events) host.addEventListener(event, reset, listenerOptions);
      const document = host.document;
      const onVisibility = (): void => {
        if (!document?.hidden) reset();
      };
      if (listenForVisibilityChange) {
        document?.addEventListener("visibilitychange", onVisibility, listenerOptions);
      }
      reset();
      onCleanup(() => {
        tracking = false;
        clear();
        for (const event of events) host.removeEventListener(event, reset);
        document?.removeEventListener("visibilitychange", onVisibility);
      });
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stop.stop());

  return { idle: readonly(idle), lastActive: readonly(lastActive), reset };
}

function browserIdleHost(): IdleHost | undefined {
  return typeof window !== "undefined" ? window : undefined;
}
