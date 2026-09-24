import { toValue } from "vue";
import type { MaybeRefOrGetter } from "vue";

import { resolveElement } from "./element-target.ts";
import type { MaybeElementTarget } from "./element-target.ts";
import { tryOnScopeDispose } from "./scope.ts";
import type { TimeoutScheduler } from "./timeout-scheduler.ts";

/** Window-like capability observed by {@link onClickOutside}. */
export interface ClickOutsideHost extends EventTarget {
  /** Document used to detect focus moving into an iframe. */
  readonly document: { readonly activeElement: Element | null };
}

/** Ignore entry: an element target or a CSS selector matched against the event path. */
export type ClickOutsideIgnore = MaybeElementTarget | string;

/** Options for {@link onClickOutside}. */
export interface OnClickOutsideOptions {
  /**
   * Elements (or selectors) whose clicks never count as outside, e.g. the
   * button that toggles a popover.
   *
   * @default []
   */
  readonly ignore?: MaybeRefOrGetter<readonly ClickOutsideIgnore[]>;

  /**
   * Listen in the capture phase so `stopPropagation()` inside the page
   * cannot hide outside clicks.
   *
   * @default true
   */
  readonly capture?: boolean;

  /**
   * Treat focus moving into an `<iframe>` outside the target as an outside
   * click (iframes swallow pointer events).
   *
   * @default false
   */
  readonly detectIframe?: boolean;

  /**
   * Owns the zero-delay timer used for iframe detection.
   *
   * @default globalThis timer functions
   */
  readonly scheduler?: TimeoutScheduler;

  /**
   * Reactive window capability for alternate runtimes and tests.
   *
   * @default globalThis.window when available
   */
  readonly host?: MaybeRefOrGetter<ClickOutsideHost | null | undefined>;
}

const defaultScheduler: TimeoutScheduler = {
  setTimeout: (callback, delayMs) => globalThis.setTimeout(callback, delayMs),
  clearTimeout: (handle) => {
    globalThis.clearTimeout(handle as ReturnType<typeof setTimeout>);
  },
};

/**
 * Call a handler when the user clicks (or focuses an iframe) outside an element.
 *
 * A click counts as outside only when its `pointerdown` also started outside
 * the target, so drags that begin inside (e.g. text selection) and end
 * outside are ignored. Event paths (`composedPath()`) make shadow-DOM targets
 * work. The host is resolved once, when called. Nothing is registered
 * during server rendering; listeners are removed with the owning scope or by
 * the returned stop function.
 *
 * @param target Reactive element target.
 * @param handler Called with the outside `PointerEvent`/`MouseEvent` or `FocusEvent`.
 * @param options Ignore list, phase, iframe detection, and capabilities.
 * @default options {}
 * @returns Stop function.
 */
export function onClickOutside(
  target: MaybeElementTarget,
  handler: (event: Event) => void,
  options: OnClickOutsideOptions = {},
): () => void {
  const host = options.host === undefined ? browserClickHost() : toValue(options.host);
  if (!host) return () => undefined;
  const capture = options.capture ?? true;
  const scheduler = options.scheduler ?? defaultScheduler;
  let startedInside = false;
  let pendingBlur: unknown;
  let hasPendingBlur = false;

  const pathOf = (event: Event): readonly EventTarget[] => {
    const path = typeof event.composedPath === "function" ? event.composedPath() : [];
    return path.length > 0 ? path : event.target ? [event.target] : [];
  };
  const isIgnored = (event: Event): boolean => {
    const path = pathOf(event);
    for (const entry of toValue(options.ignore) ?? []) {
      if (typeof entry === "string") {
        if (path.some((node) => matchesSelector(node, entry))) return true;
        continue;
      }
      const element = resolveElement(entry);
      if (element && path.some((node) => node === element || containsNode(element, node))) {
        return true;
      }
    }
    return false;
  };
  const isInside = (event: Event): boolean => {
    const element = resolveElement(target);
    if (!element) return true;
    return pathOf(event).some((node) => node === element || containsNode(element, node));
  };

  const onPointerDown = (event: Event): void => {
    startedInside = isInside(event) || isIgnored(event);
  };
  const onClick = (event: Event): void => {
    const inside = isInside(event) || isIgnored(event);
    const began = startedInside;
    startedInside = false;
    if (inside || began) return;
    handler(event);
  };
  const onBlur = (event: Event): void => {
    if (hasPendingBlur) scheduler.clearTimeout(pendingBlur);
    hasPendingBlur = true;
    pendingBlur = scheduler.setTimeout(() => {
      hasPendingBlur = false;
      const active = host.document.activeElement;
      const element = resolveElement(target);
      if (active?.tagName === "IFRAME" && element && !containsNode(element, active)) {
        handler(event);
      }
    }, 0);
  };

  host.addEventListener("pointerdown", onPointerDown, { capture, passive: true });
  host.addEventListener("click", onClick, { capture, passive: true });
  if (options.detectIframe) host.addEventListener("blur", onBlur);

  let stopped = false;
  const stop = (): void => {
    if (stopped) return;
    stopped = true;
    host.removeEventListener("pointerdown", onPointerDown, { capture });
    host.removeEventListener("click", onClick, { capture });
    host.removeEventListener("blur", onBlur);
    if (hasPendingBlur) scheduler.clearTimeout(pendingBlur);
  };
  tryOnScopeDispose(stop);
  return stop;
}

function containsNode(element: Element, node: EventTarget): boolean {
  return typeof element.contains === "function" && isNodeLike(node) && element.contains(node);
}

function isNodeLike(value: EventTarget): value is Node {
  return "nodeType" in value;
}

function matchesSelector(node: EventTarget, selector: string): boolean {
  if (!("matches" in node) || typeof node.matches !== "function") return false;
  try {
    return node.matches(selector) === true;
  } catch {
    return false;
  }
}

function browserClickHost(): ClickOutsideHost | undefined {
  return typeof window !== "undefined" ? window : undefined;
}
