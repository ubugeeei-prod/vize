import {
  computed,
  hasInjectionContext,
  readonly,
  ref,
  shallowRef,
  toValue,
  watch,
  watchPostEffect,
} from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";
import type { IntervalScheduler } from "./use-interval.ts";

/** Minimal `navigator.userActivation` (`UserActivation`). */
export interface UserActivationLike {
  /** Whether the page ever had sticky activation. */
  readonly hasBeenActive: boolean;
  /** Whether the page currently has transient activation. */
  readonly isActive: boolean;
}

/** Options for {@link useUserActivation}. */
export interface UseUserActivationOptions {
  /**
   * User activation state for alternate runtimes and tests.
   *
   * @default window.navigator.userActivation when it exists
   */
  readonly userActivation?: MaybeRefOrGetter<UserActivationLike | null | undefined>;

  /**
   * Target whose activation-triggering events refresh the state.
   *
   * @default window
   */
  readonly target?: MaybeRefOrGetter<EventTarget | null | undefined>;

  /**
   * Events that refresh the state.
   *
   * @default ["keydown", "mousedown", "pointerdown", "pointerup", "touchend"]
   */
  readonly events?: readonly string[];

  /**
   * Poll interval in milliseconds while `isActive` is true, so its expiry is
   * observed. `0` disables polling.
   *
   * @default 1000
   */
  readonly interval?: number;

  /**
   * Timer host for polling.
   *
   * @default window timer functions
   */
  readonly scheduler?: IntervalScheduler;
}

/** Reactive state returned by {@link useUserActivation}. */
export interface UserActivationControls {
  /** Whether the User Activation API is available. */
  readonly supported: ComputedRef<boolean>;
  /** Sticky activation: the user interacted with the page at least once. */
  readonly hasBeenActive: Readonly<Ref<boolean>>;
  /** Transient activation: a gated API (popup, fullscreen, clipboard) may be called now. */
  readonly isActive: Readonly<Ref<boolean>>;
  /** Re-read the activation state. */
  readonly refresh: () => void;
  /** Remove listeners and timers. Idempotent. */
  readonly stop: () => void;
}

const defaultEvents: readonly string[] = [
  "keydown",
  "mousedown",
  "pointerdown",
  "pointerup",
  "touchend",
];

function isUserActivation(candidate: unknown): candidate is UserActivationLike {
  return typeof candidate === "object" && candidate !== null && "isActive" in candidate;
}

function browserUserActivation(): UserActivationLike | undefined {
  if (typeof window === "undefined") return undefined;
  const candidate: unknown = Reflect.get(window.navigator, "userActivation");
  return isUserActivation(candidate) ? candidate : undefined;
}

function browserScheduler(): IntervalScheduler | undefined {
  if (typeof window === "undefined") return undefined;
  return {
    setInterval: (callback, intervalMs) => window.setInterval(callback, intervalMs),
    clearInterval: (handle) => {
      if (typeof handle === "number") window.clearInterval(handle);
    },
  };
}

/**
 * Track sticky and transient user activation with the User Activation API.
 *
 * The state is refreshed on activation-triggering events of `target`
 * (immediately and again after a microtask) and polled while `isActive` is
 * true so its expiry is observed. Listeners and the timer are removed when
 * the owning reactive scope stops; outside a scope call `stop()`.
 *
 * Server rendering: no listeners or timers, `supported`, `hasBeenActive`,
 * and `isActive` are false.
 *
 * @example
 * ```ts
 * const { isActive } = useUserActivation();
 * const canOpenPopup = computed(() => isActive.value);
 * ```
 *
 * @param options Capability host, event target, and polling policy.
 * @default options {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_USER_ACTIVATION_INVALID_INTERVAL` when `interval` is negative or not finite.
 * @returns Activation state.
 */
export function useUserActivation(options: UseUserActivationOptions = {}): UserActivationControls {
  const interval = options.interval ?? 1_000;
  if (!Number.isFinite(interval) || interval < 0) {
    throw new RangeError(
      `[VIZE_COMPOSE_USER_ACTIVATION_INVALID_INTERVAL] interval must be a finite, non-negative number; received ${String(interval)}`,
    );
  }
  const hasBeenActive = ref(false);
  const isActive = ref(false);
  let timer: { scheduler: IntervalScheduler; handle: unknown } | undefined;

  // Inside a component the host resolves only after mount (a post-flush
  // job), so a hydrating client first renders the same unsupported state as
  // the server. Outside components it resolves immediately.
  const hydrated = shallowRef(!hasInjectionContext());
  if (!hydrated.value) {
    watchPostEffect(() => {
      hydrated.value = true;
    });
  }
  const resolveHost = (): UserActivationLike | undefined =>
    !hydrated.value
      ? undefined
      : options.userActivation === undefined
        ? browserUserActivation()
        : (toValue(options.userActivation) ?? undefined);

  const stopPolling = (): void => {
    timer?.scheduler.clearInterval(timer.handle);
    timer = undefined;
  };

  const refresh = (): void => {
    const host = resolveHost();
    hasBeenActive.value = host?.hasBeenActive ?? false;
    isActive.value = host?.isActive ?? false;
    if (!isActive.value) {
      stopPolling();
      return;
    }
    if (timer || interval === 0) return;
    const scheduler = options.scheduler ?? browserScheduler();
    if (scheduler) timer = { scheduler, handle: scheduler.setInterval(refresh, interval) };
  };

  const onEvent = (): void => {
    refresh();
    void Promise.resolve().then(refresh);
  };

  const stopWatch = watch(
    () =>
      [
        options.target === undefined
          ? typeof window === "undefined"
            ? undefined
            : window
          : (toValue(options.target) ?? undefined),
        resolveHost(),
      ] as const,
    ([target, host], _previous, onCleanup) => {
      // Read the state once the host resolves (after mount inside a component).
      if (host) refresh();
      if (!target || !host) return;
      const events = options.events ?? defaultEvents;
      for (const type of events)
        target.addEventListener(type, onEvent, { capture: true, passive: true });
      onCleanup(() => {
        for (const type of events) target.removeEventListener(type, onEvent, { capture: true });
      });
    },
    { immediate: true, flush: "sync" },
  );

  const stop = (): void => {
    stopWatch();
    stopPolling();
  };

  tryOnScopeDispose(stop);

  return {
    supported: computed(() => resolveHost() !== undefined),
    hasBeenActive: readonly(hasBeenActive),
    isActive: readonly(isActive),
    refresh,
    stop,
  };
}
