import { readonly, ref, toValue, watch } from "vue";
import type { MaybeRefOrGetter, Ref } from "vue";

import type { MediaQueryHost } from "./media-query.ts";
import { tryOnScopeDispose } from "./scope.ts";

/** Window-like capability observed by {@link useDevicePixelRatio}. */
export interface DevicePixelRatioHost extends MediaQueryHost {
  /** Ratio of physical to CSS pixels. */
  readonly devicePixelRatio: number;
}

/** Options for {@link useDevicePixelRatio}. */
export interface UseDevicePixelRatioOptions {
  /**
   * Ratio exposed during server rendering.
   *
   * @default 1
   */
  readonly ssrPixelRatio?: number;

  /**
   * Reactive window capability for alternate runtimes and tests.
   *
   * @default globalThis.window when available
   */
  readonly host?: MaybeRefOrGetter<DevicePixelRatioHost | null | undefined>;
}

/** Reactive ratio returned by {@link useDevicePixelRatio}. */
export interface DevicePixelRatioControls {
  /** Current device pixel ratio. */
  readonly pixelRatio: Readonly<Ref<number>>;
  /** Whether a window capability is attached. `false` during server rendering. */
  readonly isSupported: Readonly<Ref<boolean>>;
}

/**
 * Track `window.devicePixelRatio` (zoom and moving between displays).
 *
 * Subscribes to a `(resolution: <ratio>dppx)` media query and re-subscribes
 * with the new ratio after each change, which is the only change signal the
 * platform offers. Server renders expose `ssrPixelRatio`.
 *
 * @param options Server fallback and window capability.
 * @default options {}
 * @returns Reactive ratio and support flag.
 */
export function useDevicePixelRatio(
  options: UseDevicePixelRatioOptions = {},
): DevicePixelRatioControls {
  const fallback = options.ssrPixelRatio ?? 1;
  const pixelRatio = ref(fallback);
  const isSupported = ref(false);

  const stop = watch(
    () => (options.host === undefined ? browserPixelRatioHost() : toValue(options.host)),
    (host, _previous, onCleanup) => {
      isSupported.value = Boolean(host);
      if (!host) {
        pixelRatio.value = fallback;
        return;
      }
      let release = (): void => undefined;
      const subscribe = (): void => {
        release();
        pixelRatio.value = host.devicePixelRatio;
        const media = host.matchMedia(`(resolution: ${host.devicePixelRatio}dppx)`);
        media.addEventListener("change", subscribe, { once: true });
        release = () => media.removeEventListener("change", subscribe);
      };
      subscribe();
      onCleanup(() => release());
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stop.stop());

  return { pixelRatio: readonly(pixelRatio), isSupported: readonly(isSupported) };
}

function browserPixelRatioHost(): DevicePixelRatioHost | undefined {
  return typeof window !== "undefined" && typeof window.matchMedia === "function"
    ? window
    : undefined;
}
