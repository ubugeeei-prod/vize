import { computed, readonly, ref, toRaw, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Browser navigation UI preference while fullscreen. */
export type FullscreenNavigationUI = "auto" | "hide" | "show";

/** Element that can enter fullscreen (standard or WebKit-prefixed). */
export interface FullscreenElementLike {
  /** Standard request. */
  requestFullscreen?(options?: { navigationUI?: FullscreenNavigationUI }): Promise<void>;

  /** Legacy WebKit request (Safari < 16.4). */
  webkitRequestFullscreen?(): void;
}

/** Document-level fullscreen capability (standard or WebKit-prefixed). */
export interface FullscreenHost extends EventTarget {
  /** Element currently in fullscreen. */
  readonly fullscreenElement?: unknown;

  /** Legacy WebKit fullscreen element. */
  readonly webkitFullscreenElement?: unknown;

  /** Whether fullscreen is allowed (for example by permissions policy). */
  readonly fullscreenEnabled?: boolean;

  /** Legacy WebKit flag. */
  readonly webkitFullscreenEnabled?: boolean;

  /** Standard exit. */
  exitFullscreen?(): Promise<void>;

  /** Legacy WebKit exit. */
  webkitExitFullscreen?(): void;

  /** Default fullscreen target. */
  readonly documentElement?: FullscreenElementLike | null;
}

/** Options for {@link useFullscreen}. */
export interface UseFullscreenOptions {
  /**
   * Document capability for alternate runtimes and tests.
   *
   * @default window.document when a browser window exists
   */
  readonly host?: MaybeRefOrGetter<FullscreenHost | null | undefined>;

  /**
   * Exit fullscreen when the owning reactive scope stops while this
   * composable's target is fullscreen.
   *
   * @default false
   */
  readonly autoExit?: boolean;

  /**
   * Navigation UI preference passed to `requestFullscreen`.
   *
   * @default "auto"
   */
  readonly navigationUI?: FullscreenNavigationUI;
}

/** Reactive state and actions returned by {@link useFullscreen}. */
export interface FullscreenControls {
  /** Whether fullscreen can be requested. */
  readonly supported: ComputedRef<boolean>;

  /** Whether this composable's target is the current fullscreen element. */
  readonly isFullscreen: Readonly<Ref<boolean>>;

  /**
   * Enter fullscreen. Browsers require a user gesture.
   *
   * @returns Whether the target is fullscreen afterwards.
   */
  readonly enter: () => Promise<boolean>;

  /**
   * Exit fullscreen when this composable's target owns it.
   *
   * @returns Whether the target is fullscreen afterwards.
   */
  readonly exit: () => Promise<boolean>;

  /**
   * Toggle fullscreen.
   *
   * @returns Whether the target is fullscreen afterwards.
   */
  readonly toggle: () => Promise<boolean>;
}

const changeEvents = ["fullscreenchange", "webkitfullscreenchange"] as const;

function browserFullscreenHost(): FullscreenHost | undefined {
  return typeof window === "undefined" ? undefined : window.document;
}

function currentElement(host: FullscreenHost): unknown {
  return host.fullscreenElement ?? host.webkitFullscreenElement ?? null;
}

/**
 * Enter, exit, and observe fullscreen for an element.
 *
 * Supports the standard Fullscreen API and the WebKit-prefixed variant.
 * `isFullscreen` is true only while *this* target is the fullscreen element
 * and follows `fullscreenchange`, so pressing Escape is reflected. Listeners
 * are removed when the owning scope stops; with `autoExit` the target also
 * leaves fullscreen then.
 *
 * Server rendering: `supported` and `isFullscreen` are false and nothing is
 * requested.
 *
 * @example
 * ```ts
 * const player = useTemplateRef<HTMLElement>("player");
 * const { toggle, isFullscreen } = useFullscreen(player);
 * ```
 *
 * @param target Reactive element; defaults to the document element.
 * @param options Capability, navigation UI, and exit policy.
 * @default options {}
 * @returns Fullscreen state and actions.
 */
export function useFullscreen(
  target?: MaybeRefOrGetter<FullscreenElementLike | null | undefined>,
  options: UseFullscreenOptions = {},
): FullscreenControls {
  const isFullscreen = ref(false);
  const resolveHost = (): FullscreenHost | undefined =>
    options.host === undefined ? browserFullscreenHost() : (toValue(options.host) ?? undefined);
  const resolveTarget = (): FullscreenElementLike | undefined =>
    target === undefined
      ? (resolveHost()?.documentElement ?? undefined)
      : (toValue(target) ?? undefined);

  const update = (): void => {
    const host = resolveHost();
    const element = resolveTarget();
    isFullscreen.value =
      host !== undefined && element !== undefined && toRaw(currentElement(host)) === toRaw(element);
  };

  const supported = computed(() => {
    const host = resolveHost();
    const element = resolveTarget();
    if (!host || !element) return false;
    const enabled = host.fullscreenEnabled ?? host.webkitFullscreenEnabled ?? true;
    return (
      enabled &&
      (typeof element.requestFullscreen === "function" ||
        typeof element.webkitRequestFullscreen === "function")
    );
  });

  const enter = async (): Promise<boolean> => {
    const element = resolveTarget();
    if (!supported.value || !element) return false;
    if (isFullscreen.value) return true;
    try {
      if (element.requestFullscreen) {
        await element.requestFullscreen({ navigationUI: options.navigationUI ?? "auto" });
      } else {
        element.webkitRequestFullscreen?.();
      }
    } catch {
      // Rejected requests (no user gesture, permissions policy) leave the
      // state unchanged; `isFullscreen` reports the outcome.
    }
    update();
    return isFullscreen.value;
  };

  const exit = async (): Promise<boolean> => {
    const host = resolveHost();
    update();
    if (!host || !isFullscreen.value) return isFullscreen.value;
    try {
      if (host.exitFullscreen) await host.exitFullscreen();
      else host.webkitExitFullscreen?.();
    } catch {
      // Exiting while the document already left fullscreen rejects; the
      // re-read below reports the real state.
    }
    update();
    return isFullscreen.value;
  };

  const toggle = (): Promise<boolean> => (isFullscreen.value ? exit() : enter());

  watch(
    [resolveHost, resolveTarget],
    ([host], _previous, onCleanup) => {
      update();
      if (!host) return;
      for (const event of changeEvents) host.addEventListener(event, update);
      onCleanup(() => {
        for (const event of changeEvents) host.removeEventListener(event, update);
      });
    },
    { immediate: true, flush: "sync" },
  );

  tryOnScopeDispose(() => {
    if (options.autoExit ?? false) void exit();
  });

  return { supported, isFullscreen: readonly(isFullscreen), enter, exit, toggle };
}
