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
 * During server rendering (or whenever no capability host resolves) the ref
 * holds the configured server value and no subscription is created. The
 * change subscription follows the reactive query and host: each
 * re-evaluation removes the previous listener, and the final listener is
 * removed when the owning reactive scope stops. Call inside an active scope
 * so the subscription is released. A host whose matcher throws propagates
 * the error to the active effect run; the browser default never throws.
 *
 * @param query Reactive media-query source.
 * @param options Runtime capability and server-rendered fallback.
 * @default options {}
 * @returns Readonly ref that is `true` while the query matches.
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
 * Shares {@link useMediaQuery} semantics: during server rendering the
 * preference is `"no-preference"` unless `ssrValue` is `true`, and the
 * underlying subscription is removed when the owning reactive scope stops.
 *
 * @param options Runtime capability and server-rendered fallback.
 * @default options {}
 * @returns Computed preference for `(prefers-reduced-motion: reduce)`.
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
