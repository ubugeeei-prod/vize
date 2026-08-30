import type { Rect, SafeAreaInsets } from "./positioner-types.ts";

/** Insets that leave the viewport untouched. */
export const zeroSafeAreaInsets: SafeAreaInsets = Object.freeze({
  bottom: 0,
  left: 0,
  right: 0,
  top: 0,
});

/**
 * The visible viewport in page coordinates.
 *
 * `visualViewport` already excludes on-screen keyboards and follows
 * pinch-zoom, so collision handling tracks what the user actually sees.
 * Server rendering (no `document`) reports an empty box.
 */
export function visualViewportRect(): Rect {
  const visual = globalThis.visualViewport;
  if (visual) {
    return {
      height: visual.height,
      width: visual.width,
      x: visual.offsetLeft,
      y: visual.offsetTop,
    };
  }
  const doc = globalThis.document?.documentElement;
  if (doc) {
    return { height: doc.clientHeight, width: doc.clientWidth, x: 0, y: 0 };
  }
  return { height: 0, width: 0, x: 0, y: 0 };
}

/** The document that owns a measured element, if any. */
export function ownerDocumentOf(element: object | null): Document | null {
  return (
    (element as { readonly ownerDocument?: Document | null } | null)?.ownerDocument ??
    globalThis.document ??
    null
  );
}

/** Shrink a viewport box by per-edge insets, never below an empty box. */
export function insetViewport(viewport: Rect, insets: SafeAreaInsets): Rect {
  return {
    height: Math.max(0, viewport.height - insets.top - insets.bottom),
    width: Math.max(0, viewport.width - insets.left - insets.right),
    x: viewport.x + Math.min(Math.max(0, insets.left), viewport.width),
    y: viewport.y + Math.min(Math.max(0, insets.top), viewport.height),
  };
}

function readPixels(value: string): number {
  return Math.max(0, Number.parseFloat(value) || 0);
}

/**
 * Read `env(safe-area-inset-*)` through a computed-style probe.
 *
 * CSS environment variables are not scriptable, so a hidden fixed probe is
 * measured and removed synchronously. Environments without a document or
 * without `env()` support report zero insets, keeping the read SSR-safe.
 */
export function readSafeAreaInsets(targetDocument?: Document | null): SafeAreaInsets {
  const doc = targetDocument ?? globalThis.document;
  const view = doc?.defaultView;
  if (!doc?.body || typeof view?.getComputedStyle !== "function") return zeroSafeAreaInsets;

  const probe = doc.createElement("div");
  probe.setAttribute("data-vize-ui", "safe-area");
  probe.setAttribute("aria-hidden", "true");
  probe.style.cssText =
    "position:fixed;top:0;left:0;visibility:hidden;pointer-events:none;" +
    "padding-top:env(safe-area-inset-top, 0px);" +
    "padding-right:env(safe-area-inset-right, 0px);" +
    "padding-bottom:env(safe-area-inset-bottom, 0px);" +
    "padding-left:env(safe-area-inset-left, 0px)";
  doc.body.append(probe);
  try {
    const style = view.getComputedStyle(probe);
    return {
      bottom: readPixels(style.paddingBottom),
      left: readPixels(style.paddingLeft),
      right: readPixels(style.paddingRight),
      top: readPixels(style.paddingTop),
    };
  } finally {
    probe.remove();
  }
}
