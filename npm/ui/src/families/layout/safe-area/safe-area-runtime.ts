import { onScopeDispose, readonly, shallowRef } from "vue";
import type { ShallowRef } from "vue";

/** One edge of the safe area. */
export type SafeAreaEdge = "bottom" | "left" | "right" | "top";

/** Every safe-area edge, in CSS order. */
export const SAFE_AREA_EDGES = Object.freeze([
  "top",
  "right",
  "bottom",
  "left",
] as const) satisfies readonly SafeAreaEdge[];

/** Measured insets in CSS pixels. */
export type SafeAreaEdgeInsets = { readonly [Edge in SafeAreaEdge]: number };

/** Controls returned by {@link useSafeAreaInsets}. */
export interface SafeAreaInsetsController {
  /** Latest insets; all zero on the server and before the first measurement. */
  readonly insets: Readonly<ShallowRef<SafeAreaEdgeInsets>>;

  /** Re-measure now (for example after toggling `viewport-fit=cover`). */
  readonly measure: () => void;

  /** Stop listening; also runs when the owning scope is disposed. */
  readonly stop: () => void;
}

/** Options for {@link useSafeAreaInsets}. */
export interface SafeAreaInsetsOptions {
  /**
   * Document to measure in.
   *
   * @default globalThis.document when available
   */
  readonly document?: Document | null;
}

const zeroInsets: SafeAreaEdgeInsets = Object.freeze({ top: 0, right: 0, bottom: 0, left: 0 });

/** Inline CSS custom properties mapping each edge to its `env()` inset. */
export const SAFE_AREA_STYLE = Object.freeze({
  "--vize-safe-area-inset-top": "env(safe-area-inset-top, 0px)",
  "--vize-safe-area-inset-right": "env(safe-area-inset-right, 0px)",
  "--vize-safe-area-inset-bottom": "env(safe-area-inset-bottom, 0px)",
  "--vize-safe-area-inset-left": "env(safe-area-inset-left, 0px)",
});

function readPixels(value: string): number {
  const parsed = Number.parseFloat(value);
  return Number.isFinite(parsed) ? parsed : 0;
}

/**
 * Measure `env(safe-area-inset-*)` as numbers (notches, home indicators,
 * rounded corners). CSS alone can consume `env()`; use this when script needs
 * the values, for example to offset a virtual keyboard toolbar.
 *
 * Instance-free and SSR-safe: the server and the hydrating render see zeros;
 * the first measurement is deferred to a microtask so it lands after
 * hydration, and resize/orientation changes re-measure. Requires
 * `<meta name="viewport" content="viewport-fit=cover">` for non-zero insets.
 */
export function useSafeAreaInsets(options: SafeAreaInsetsOptions = {}): SafeAreaInsetsController {
  const insets = shallowRef<SafeAreaEdgeInsets>(zeroInsets);
  const doc =
    options.document === undefined
      ? typeof document === "undefined"
        ? null
        : document
      : options.document;
  let probe: HTMLElement | undefined;
  let stopped = false;

  function measure(): void {
    if (doc === null || stopped) return;
    if (probe === undefined) {
      probe = doc.createElement("div");
      probe.setAttribute("aria-hidden", "true");
      probe.setAttribute("data-vize-ui", "safe-area-probe");
      probe.style.cssText =
        "position:fixed;visibility:hidden;pointer-events:none;inset:0 auto auto 0;width:0;height:0;" +
        SAFE_AREA_EDGES.map((edge) => `padding-${edge}:env(safe-area-inset-${edge},0px)`).join(";");
      doc.body.append(probe);
    }
    const style = doc.defaultView?.getComputedStyle(probe);
    if (style === undefined) return;
    const next: SafeAreaEdgeInsets = {
      top: readPixels(style.paddingTop),
      right: readPixels(style.paddingRight),
      bottom: readPixels(style.paddingBottom),
      left: readPixels(style.paddingLeft),
    };
    const previous = insets.value;
    if (SAFE_AREA_EDGES.some((edge) => previous[edge] !== next[edge]))
      insets.value = Object.freeze(next);
  }

  const view = doc?.defaultView ?? null;
  const onChange = () => measure();
  if (view !== null) {
    queueMicrotask(measure);
    view.addEventListener("resize", onChange, { passive: true });
    view.addEventListener("orientationchange", onChange, { passive: true });
  }

  function stop(): void {
    if (stopped) return;
    stopped = true;
    view?.removeEventListener("resize", onChange);
    view?.removeEventListener("orientationchange", onChange);
    probe?.remove();
    probe = undefined;
  }

  onScopeDispose(stop, true);
  return { insets: readonly(insets), measure, stop };
}
