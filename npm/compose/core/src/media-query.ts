import { computed, readonly, ref, toValue, watchEffect } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref } from "vue";

/** Capability required to evaluate media queries. */
export interface MediaQueryHost {
  /** Create an observable result for a media query. */
  readonly matchMedia: (query: string) => MediaQueryList;
}

/** Options for {@link useMediaQuery}. */
export interface UseMediaQueryOptions {
  /**
   * Value exposed when no media-query capability is available.
   *
   * @default false
   */
  readonly ssrValue?: boolean;

  /**
   * Reactive media-query capability for alternate runtimes and tests.
   *
   * @default globalThis.window when available
   */
  readonly host?: MaybeRefOrGetter<MediaQueryHost | null | undefined>;
}

/**
 * Evaluate a reactive media query without requiring browser globals.
 *
 * @param query Reactive media-query source.
 * @param options Runtime capability and server-rendered fallback.
 * @default options {}
 */
export function useMediaQuery(
  query: MaybeRefOrGetter<string>,
  options: UseMediaQueryOptions = {},
): Readonly<Ref<boolean>> {
  const matches = ref(options.ssrValue ?? false);

  watchEffect((onCleanup) => {
    const host = options.host === undefined ? browserMediaQueryHost() : toValue(options.host);
    if (!host) {
      matches.value = options.ssrValue ?? false;
      return;
    }

    const media = host.matchMedia(toValue(query));
    const update = (): void => {
      matches.value = media.matches;
    };
    update();
    media.addEventListener("change", update);
    onCleanup(() => media.removeEventListener("change", update));
  });

  return readonly(matches);
}

/** User motion preference exposed by {@link useReducedMotion}. */
export type MotionPreference = "reduce" | "no-preference";

/**
 * Return the reactive user motion preference.
 *
 * @param options Runtime capability and server-rendered fallback.
 * @default options {}
 */
export function useReducedMotion(
  options: UseMediaQueryOptions = {},
): ComputedRef<MotionPreference> {
  const reduced = useMediaQuery("(prefers-reduced-motion: reduce)", options);
  return computed(() => (reduced.value ? "reduce" : "no-preference"));
}

function browserMediaQueryHost(): MediaQueryHost | undefined {
  return typeof window !== "undefined" && typeof window.matchMedia === "function"
    ? window
    : undefined;
}
