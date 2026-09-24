import { readonly, ref, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Window-like capability observed by {@link useWindowSize}. */
export interface WindowSizeHost extends EventTarget {
  /** Viewport width including the vertical scrollbar. */
  readonly innerWidth: number;
  /** Viewport height including the horizontal scrollbar. */
  readonly innerHeight: number;
  /** Outer browser window width. */
  readonly outerWidth: number;
  /** Outer browser window height. */
  readonly outerHeight: number;
  /** Document whose root element reports the scrollbar-free viewport. */
  readonly document: {
    readonly documentElement: { readonly clientWidth: number; readonly clientHeight: number };
  };
}

/** Which window dimensions {@link useWindowSize} reports. */
export type WindowSizeKind = "inner" | "outer";

/** Options for {@link useWindowSize}. */
export interface UseWindowSizeOptions {
  /**
   * Width exposed during server rendering and before hydration.
   *
   * @default 0
   */
  readonly initialWidth?: number;

  /**
   * Height exposed during server rendering and before hydration.
   *
   * @default 0
   */
  readonly initialHeight?: number;

  /**
   * Include scrollbars in the inner dimensions. When `false`, the root
   * element's `clientWidth`/`clientHeight` are reported instead.
   *
   * @default true
   */
  readonly includeScrollbar?: boolean;

  /**
   * Report inner (viewport) or outer (browser window) dimensions.
   *
   * @default "inner"
   */
  readonly type?: WindowSizeKind;

  /**
   * Also re-measure on `orientationchange` for engines that resize late.
   *
   * @default true
   */
  readonly listenOrientation?: boolean;

  /**
   * Reactive window capability for alternate runtimes and tests.
   *
   * @default globalThis.window when available
   */
  readonly host?: MaybeRefOrGetter<WindowSizeHost | null | undefined>;
}

/** Reactive size returned by {@link useWindowSize}. */
export interface WindowSizeControls {
  /** Current width in CSS pixels. */
  readonly width: Readonly<Ref<number>>;
  /** Current height in CSS pixels. */
  readonly height: Readonly<Ref<number>>;
  /** Whether a window capability is attached. `false` during server rendering. */
  readonly isSupported: Readonly<Ref<boolean>>;
}

/**
 * Track the window size reactively.
 *
 * Server renders expose `initialWidth`/`initialHeight` (default `0`), so the
 * first client render matches; the real size is applied as soon as a window
 * resolves. Resize (and orientation) listeners are passive and removed with
 * the owning reactive scope.
 *
 * @param options Initial size, measurement kind, and capability.
 * @default options {}
 * @returns Reactive width/height plus a support flag.
 */
export function useWindowSize(options: UseWindowSizeOptions = {}): WindowSizeControls {
  const {
    initialWidth = 0,
    initialHeight = 0,
    includeScrollbar = true,
    type = "inner",
    listenOrientation = true,
  } = options;
  const width = ref(initialWidth);
  const height = ref(initialHeight);
  const isSupported = ref(false);

  const stopWatch = watch(
    () => (options.host === undefined ? browserWindowSizeHost() : toValue(options.host)),
    (host, _previous, onCleanup) => {
      isSupported.value = Boolean(host);
      if (!host) {
        width.value = initialWidth;
        height.value = initialHeight;
        return;
      }
      const update = (): void => {
        if (type === "outer") {
          width.value = host.outerWidth;
          height.value = host.outerHeight;
        } else if (includeScrollbar) {
          width.value = host.innerWidth;
          height.value = host.innerHeight;
        } else {
          width.value = host.document.documentElement.clientWidth;
          height.value = host.document.documentElement.clientHeight;
        }
      };
      update();
      const listenerOptions: AddEventListenerOptions = { passive: true };
      host.addEventListener("resize", update, listenerOptions);
      if (listenOrientation) host.addEventListener("orientationchange", update, listenerOptions);
      onCleanup(() => {
        host.removeEventListener("resize", update);
        if (listenOrientation) host.removeEventListener("orientationchange", update);
      });
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stopWatch.stop());

  return { width: readonly(width), height: readonly(height), isSupported: readonly(isSupported) };
}

function browserWindowSizeHost(): WindowSizeHost | undefined {
  return typeof window !== "undefined" ? window : undefined;
}
