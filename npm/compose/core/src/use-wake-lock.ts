import { computed, readonly, ref, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Wake lock types defined by the Screen Wake Lock API. */
export type WakeLockKind = "screen";

/** Minimal `WakeLockSentinel` consumed by {@link useWakeLock}. */
export interface WakeLockSentinelLike extends EventTarget {
  /** Whether the platform already released this lock. */
  readonly released: boolean;

  /** Release the lock. */
  release(): Promise<void>;
}

/** Capabilities used by {@link useWakeLock}. */
export interface WakeLockHost {
  /** Screen Wake Lock API (`navigator.wakeLock`). */
  readonly wakeLock: {
    /** Request a lock of `type`. */
    request(type: WakeLockKind): Promise<WakeLockSentinelLike>;
  };

  /**
   * Document whose `visibilitychange` re-acquires a lock the platform
   * released while the page was hidden.
   */
  readonly document?: {
    /** Current visibility. */
    readonly visibilityState: string;
    /** Subscribe to `visibilitychange`. */
    addEventListener(type: "visibilitychange", listener: () => void): void;
    /** Unsubscribe from `visibilitychange`. */
    removeEventListener(type: "visibilitychange", listener: () => void): void;
  } | null;
}

/** Options for {@link useWakeLock}. */
export interface UseWakeLockOptions {
  /**
   * Wake Lock capability for alternate runtimes and tests.
   *
   * @default window.navigator and window.document when `navigator.wakeLock` exists
   */
  readonly host?: MaybeRefOrGetter<WakeLockHost | null | undefined>;

  /**
   * Re-acquire a requested lock when the page becomes visible again (the
   * platform releases screen locks whenever the page is hidden).
   *
   * @default true
   */
  readonly reacquireOnVisible?: boolean;
}

/** Reactive state and actions returned by {@link useWakeLock}. */
export interface WakeLockControls {
  /** Whether the Screen Wake Lock API is available. */
  readonly supported: ComputedRef<boolean>;

  /** Whether a lock is currently held. */
  readonly active: Readonly<Ref<boolean>>;

  /**
   * Whether the caller wants a lock: set by `request`, cleared by `release`.
   * While requested, the lock is re-acquired after visibility returns.
   */
  readonly requested: Readonly<Ref<boolean>>;

  /** Most recent request failure (for example NotAllowedError), cleared on success. */
  readonly error: Readonly<ShallowRef<unknown>>;

  /**
   * Request a wake lock.
   *
   * @param type Lock type.
   * @default type "screen"
   * @returns Whether a lock is held afterwards.
   */
  readonly request: (type?: WakeLockKind) => Promise<boolean>;

  /**
   * Release the lock and stop re-acquiring it. Repeated calls are safe.
   *
   * @returns Resolves once the lock was released.
   */
  readonly release: () => Promise<void>;
}

function browserWakeLockHost(): WakeLockHost | undefined {
  if (typeof window === "undefined") return undefined;
  const { navigator, document } = window;
  return "wakeLock" in navigator && navigator.wakeLock
    ? { wakeLock: navigator.wakeLock, document }
    : undefined;
}

/**
 * Keep the screen awake with the Screen Wake Lock API.
 *
 * Browsers release screen locks when the page is hidden; while a lock is
 * `requested`, it is re-acquired on the next `visibilitychange` to visible.
 * Request failures (missing user activation, battery saver) are exposed
 * through `error` rather than thrown. The lock and the visibility listener
 * are released when the owning reactive scope stops.
 *
 * Server rendering: nothing is requested, `supported` and `active` are false.
 *
 * @example
 * ```ts
 * const wakeLock = useWakeLock();
 * const startPresentation = () => wakeLock.request();
 * ```
 *
 * @param options Capability host and re-acquisition policy.
 * @default options {}
 * @returns Wake lock state and actions.
 */
export function useWakeLock(options: UseWakeLockOptions = {}): WakeLockControls {
  const active = ref(false);
  const requested = ref(false);
  const error = shallowRef<unknown>(undefined);
  let sentinel: WakeLockSentinelLike | undefined;
  let kind: WakeLockKind = "screen";
  let generation = 0;

  const resolveHost = (): WakeLockHost | undefined =>
    options.host === undefined ? browserWakeLockHost() : (toValue(options.host) ?? undefined);

  const onRelease = (): void => {
    active.value = false;
    sentinel = undefined;
  };

  const acquire = async (): Promise<boolean> => {
    const host = resolveHost();
    if (!host) return false;
    const current = ++generation;
    try {
      const next = await host.wakeLock.request(kind);
      if (current !== generation || !requested.value) {
        await next.release();
        return active.value;
      }
      sentinel?.removeEventListener("release", onRelease);
      sentinel = next;
      next.addEventListener("release", onRelease, { once: true });
      active.value = !next.released;
      error.value = undefined;
      return active.value;
    } catch (cause) {
      if (current === generation) error.value = cause;
      return false;
    }
  };

  const request = async (type: WakeLockKind = "screen"): Promise<boolean> => {
    kind = type;
    requested.value = true;
    if (active.value) return true;
    return acquire();
  };

  const release = async (): Promise<void> => {
    requested.value = false;
    generation += 1;
    const current = sentinel;
    sentinel = undefined;
    active.value = false;
    if (!current) return;
    current.removeEventListener("release", onRelease);
    if (!current.released) await current.release();
  };

  if (options.reacquireOnVisible ?? true) {
    watch(
      () => resolveHost()?.document ?? undefined,
      (document, _previous, onCleanup) => {
        if (!document) return;
        const onVisibility = (): void => {
          if (document.visibilityState === "visible" && requested.value && !active.value) {
            void acquire();
          }
        };
        document.addEventListener("visibilitychange", onVisibility);
        onCleanup(() => document.removeEventListener("visibilitychange", onVisibility));
      },
      { immediate: true, flush: "sync" },
    );
  }

  tryOnScopeDispose(() => {
    void release();
  });

  return {
    supported: computed(() => resolveHost() !== undefined),
    active: readonly(active),
    requested: readonly(requested),
    error: readonly(error),
    request,
    release,
  };
}
