import { computed, readonly, ref, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref, ShallowRef } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Available (work) area of a screen in multi-screen coordinates. */
export interface ScreenArea {
  /** Left edge of the available area. */
  readonly availLeft: number;
  /** Top edge of the available area. */
  readonly availTop: number;
  /** Width of the available area. */
  readonly availWidth: number;
  /** Height of the available area. */
  readonly availHeight: number;
}

/** Plain snapshot of a `ScreenDetailed`. */
export interface ScreenSnapshot extends ScreenArea {
  /** Human-readable label, possibly empty. */
  readonly label: string;
  /** Left edge in multi-screen coordinates. */
  readonly left: number;
  /** Top edge in multi-screen coordinates. */
  readonly top: number;
  /** Full width. */
  readonly width: number;
  /** Full height. */
  readonly height: number;
  /** Device pixel ratio. */
  readonly devicePixelRatio: number;
  /** Color depth in bits. */
  readonly colorDepth: number;
  /** Whether this is the primary screen. */
  readonly isPrimary: boolean;
  /** Whether this screen is built into the device. */
  readonly isInternal: boolean;
}

/** Minimal `ScreenDetailed`. */
export interface ScreenDetailedLike extends ScreenSnapshot, EventTarget {}

/** Minimal `ScreenDetails`. */
export interface ScreenDetailsLike extends EventTarget {
  /** Every connected screen. */
  readonly screens: readonly ScreenDetailedLike[];
  /** Screen hosting the current window. */
  readonly currentScreen: ScreenDetailedLike;
}

/** Capabilities used by {@link useScreenDetails}. */
export interface ScreenDetailsHost {
  /** Window Management entry point (permission-gated). */
  getScreenDetails?(): Promise<ScreenDetailsLike>;
  /**
   * `window.screen`. Its `isExtended` flag is read and, when it is an
   * `EventTarget`, its `change` event refreshes the flag.
   */
  readonly screen?: object | null;
  /** `window.open`, used by `openOnScreen`. */
  open?(url: string, target: string, features: string): Window | null;
}

/** Options for {@link useScreenDetails}. */
export interface UseScreenDetailsOptions {
  /**
   * Window Management capability for alternate runtimes and tests.
   *
   * @default window
   */
  readonly host?: MaybeRefOrGetter<ScreenDetailsHost | null | undefined>;
}

/** Desired window geometry relative to a screen's available area. */
export interface WindowPlacementOptions {
  /**
   * Window width, clamped to the available width.
   *
   * @default the available width
   */
  readonly width?: number;
  /**
   * Window height, clamped to the available height.
   *
   * @default the available height
   */
  readonly height?: number;
  /**
   * Offset from the available area's left edge.
   *
   * @default centered horizontally
   */
  readonly left?: number;
  /**
   * Offset from the available area's top edge.
   *
   * @default centered vertically
   */
  readonly top?: number;
}

/** Absolute window geometry for `window.open` features or `moveTo`/`resizeTo`. */
export interface WindowPlacement {
  /** Absolute left. */
  readonly left: number;
  /** Absolute top. */
  readonly top: number;
  /** Width. */
  readonly width: number;
  /** Height. */
  readonly height: number;
}

/** Options of {@link ScreenDetailsControls.openOnScreen}. */
export interface OpenOnScreenOptions extends WindowPlacementOptions {
  /**
   * Browsing context name.
   *
   * @default "_blank"
   */
  readonly target?: string;
  /**
   * Extra comma-separated window features, for example `"popup"`.
   *
   * @default ""
   */
  readonly features?: string;
}

/** Discriminated outcome of {@link ScreenDetailsControls.request}. */
export type ScreenDetailsResult =
  | {
      /** Permission granted; state is live. */
      readonly status: "granted";
      /** Screen snapshots. */
      readonly screens: readonly ScreenSnapshot[];
    }
  | {
      /** `denied`: permission refused; `unsupported`: no API; `failed`: anything else. */
      readonly status: "denied" | "unsupported" | "failed";
      /** Error thrown by the host, when one was thrown. */
      readonly error: unknown;
    };

/** Reactive state and actions returned by {@link useScreenDetails}. */
export interface ScreenDetailsControls {
  /** Whether the Window Management API is available. */
  readonly supported: ComputedRef<boolean>;
  /** Whether the window spans a multi-screen setup (`screen.isExtended`). */
  readonly isExtended: Readonly<Ref<boolean>>;
  /** Snapshots of every screen, empty until `request` succeeds. */
  readonly screens: Readonly<ShallowRef<readonly ScreenSnapshot[]>>;
  /** Snapshot of the current screen, once granted. */
  readonly currentScreen: Readonly<ShallowRef<ScreenSnapshot | undefined>>;
  /** Whether `request` is in progress. */
  readonly pending: Readonly<Ref<boolean>>;
  /** Most recent request failure, cleared on success. */
  readonly error: Readonly<ShallowRef<unknown>>;
  /**
   * Request screen details (prompts for the window-management permission).
   *
   * @returns The discriminated outcome; never rejects.
   */
  readonly request: () => Promise<ScreenDetailsResult>;
  /**
   * Open a window placed on `screen`.
   *
   * @param url URL to open.
   * @param screen Target screen (a snapshot or any {@link ScreenArea}).
   * @param options Placement, target name, and extra features.
   * @returns The opened window, or `null` when blocked or unsupported.
   */
  readonly openOnScreen: (
    url: string,
    screen: ScreenArea,
    options?: OpenOnScreenOptions,
  ) => Window | null;
  /** Remove every listener. Idempotent. */
  readonly stop: () => void;
}

/**
 * Snapshot a `ScreenDetailed` into a plain object.
 *
 * @param screen Screen to copy.
 * @returns Plain snapshot.
 */
export function snapshotScreen(screen: ScreenSnapshot): ScreenSnapshot {
  return {
    label: screen.label,
    left: screen.left,
    top: screen.top,
    width: screen.width,
    height: screen.height,
    availLeft: screen.availLeft,
    availTop: screen.availTop,
    availWidth: screen.availWidth,
    availHeight: screen.availHeight,
    devicePixelRatio: screen.devicePixelRatio,
    colorDepth: screen.colorDepth,
    isPrimary: screen.isPrimary,
    isInternal: screen.isInternal,
  };
}

/**
 * Compute absolute window geometry inside a screen's available area.
 *
 * Sizes are clamped to the available area; omitted offsets center the window.
 *
 * @param screen Target screen area.
 * @param placement Desired size and offsets.
 * @default placement {}
 * @throws `RangeError` tagged `VIZE_COMPOSE_SCREEN_DETAILS_INVALID_PLACEMENT` for negative or non-finite values.
 * @returns Absolute placement.
 */
export function placeWindow(
  screen: ScreenArea,
  placement: WindowPlacementOptions = {},
): WindowPlacement {
  for (const value of [placement.width, placement.height, placement.left, placement.top]) {
    if (value !== undefined && (!Number.isFinite(value) || value < 0)) {
      throw new RangeError(
        `[VIZE_COMPOSE_SCREEN_DETAILS_INVALID_PLACEMENT] placement values must be finite and non-negative; received ${String(value)}`,
      );
    }
  }
  const width = Math.round(Math.min(placement.width ?? screen.availWidth, screen.availWidth));
  const height = Math.round(Math.min(placement.height ?? screen.availHeight, screen.availHeight));
  const left = placement.left ?? (screen.availWidth - width) / 2;
  const top = placement.top ?? (screen.availHeight - height) / 2;
  return {
    left: Math.round(screen.availLeft + Math.min(left, screen.availWidth - width)),
    top: Math.round(screen.availTop + Math.min(top, screen.availHeight - height)),
    width,
    height,
  };
}

function browserHost(): ScreenDetailsHost | undefined {
  return typeof window === "undefined" ? undefined : window;
}

function errorName(error: unknown): unknown {
  return typeof error === "object" && error !== null && "name" in error ? error.name : undefined;
}

/**
 * Enumerate and target multiple screens with the Window Management API.
 *
 * `isExtended` is live from `window.screen`. `request()` asks for the
 * window-management permission; once granted, `screens` and
 * `currentScreen` stay current through `screenschange`,
 * `currentscreenchange`, and per-screen `change` events. `openOnScreen`
 * opens a window placed with {@link placeWindow}. Listeners are removed when
 * the owning reactive scope stops; outside a scope call `stop()`.
 *
 * Server rendering: nothing is requested, `supported` and `isExtended` are
 * false and `screens` is empty.
 *
 * @example
 * ```ts
 * const { request, screens, openOnScreen } = useScreenDetails();
 * await request();
 * const external = screens.value.find((screen) => !screen.isPrimary);
 * if (external) openOnScreen("/slides", external, { features: "popup" });
 * ```
 *
 * @param options Capability host.
 * @default options {}
 * @returns Screen state and actions.
 */
export function useScreenDetails(options: UseScreenDetailsOptions = {}): ScreenDetailsControls {
  const isExtended = ref(false);
  const screens = shallowRef<readonly ScreenSnapshot[]>([]);
  const currentScreen = shallowRef<ScreenSnapshot | undefined>(undefined);
  const pending = ref(false);
  const error = shallowRef<unknown>(undefined);
  let unbind: (() => void) | undefined;
  let stopped = false;

  const resolveHost = (): ScreenDetailsHost | undefined =>
    options.host === undefined ? browserHost() : (toValue(options.host) ?? undefined);

  const bind = (details: ScreenDetailsLike): void => {
    unbind?.();
    let watched: readonly ScreenDetailedLike[] = [];
    const sync = (): void => {
      for (const screen of watched) screen.removeEventListener("change", sync);
      watched = details.screens;
      for (const screen of watched) screen.addEventListener("change", sync);
      screens.value = watched.map(snapshotScreen);
      currentScreen.value = snapshotScreen(details.currentScreen);
    };
    sync();
    details.addEventListener("screenschange", sync);
    details.addEventListener("currentscreenchange", sync);
    unbind = () => {
      for (const screen of watched) screen.removeEventListener("change", sync);
      details.removeEventListener("screenschange", sync);
      details.removeEventListener("currentscreenchange", sync);
    };
  };

  const request = async (): Promise<ScreenDetailsResult> => {
    const host = resolveHost();
    if (!host?.getScreenDetails) return { status: "unsupported", error: undefined };
    pending.value = true;
    try {
      const details = await host.getScreenDetails();
      if (stopped) return { status: "failed", error: undefined };
      bind(details);
      error.value = undefined;
      return { status: "granted", screens: screens.value };
    } catch (cause) {
      error.value = cause;
      return { status: errorName(cause) === "NotAllowedError" ? "denied" : "failed", error: cause };
    } finally {
      pending.value = false;
    }
  };

  const openOnScreen = (
    url: string,
    screen: ScreenArea,
    openOptions: OpenOnScreenOptions = {},
  ): Window | null => {
    const host = resolveHost();
    if (!host?.open) return null;
    const { left, top, width, height } = placeWindow(screen, openOptions);
    const features = [`left=${left},top=${top},width=${width},height=${height}`];
    if (openOptions.features) features.push(openOptions.features);
    return host.open(url, openOptions.target ?? "_blank", features.join(","));
  };

  const stopWatch = watch(
    () => resolveHost()?.screen ?? undefined,
    (screen, _previous, onCleanup) => {
      const sync = (): void => {
        isExtended.value =
          screen !== undefined && "isExtended" in screen && screen.isExtended === true;
      };
      sync();
      if (!(screen instanceof EventTarget)) return;
      screen.addEventListener("change", sync);
      onCleanup(() => screen.removeEventListener("change", sync));
    },
    { immediate: true, flush: "sync" },
  );

  const stop = (): void => {
    stopped = true;
    stopWatch();
    unbind?.();
    unbind = undefined;
  };

  tryOnScopeDispose(stop);

  return {
    supported: computed(() => typeof resolveHost()?.getScreenDetails === "function"),
    isExtended: readonly(isExtended),
    screens: readonly(screens),
    currentScreen: readonly(currentScreen),
    pending: readonly(pending),
    error: readonly(error),
    request,
    openOnScreen,
    stop,
  };
}
