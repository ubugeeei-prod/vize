import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef } from "vue";

import type {
  SizeObserverController,
  SizeObserverEntry,
  SizeObserverOptions,
  VisibilityObserverController,
  VisibilityObserverEntry,
  VisibilityObserverOptions,
} from "./measure-types.ts";

const invalidOptionDiagnostic = "VIZE_UI_MEASURE_OPTION";
const disposedDiagnostic = "VIZE_UI_MEASURE_DISPOSED";
const setupDiagnostic = "VIZE_UI_MEASURE_SETUP";
const boxes = new Set(["border-box", "content-box"]);

function assertCallback(name: string, value: unknown): asserts value is (...args: never) => void {
  if (typeof value !== "function") {
    throw new TypeError(`${invalidOptionDiagnostic}: ${name} must be a function`);
  }
}

function assertElement(target: Element): void {
  if (!target || target.nodeType !== 1) {
    throw new TypeError(`${invalidOptionDiagnostic}: observation targets must be elements`);
  }
}

function readBoxSize(entry: ResizeObserverEntry, box: "border-box" | "content-box") {
  const boxSize = box === "border-box" ? entry.borderBoxSize : entry.contentBoxSize;
  const size = Array.isArray(boxSize) ? boxSize[0] : undefined;
  if (size && typeof size.inlineSize === "number" && typeof size.blockSize === "number") {
    return { width: size.inlineSize, height: size.blockSize };
  }
  return { width: entry.contentRect.width, height: entry.contentRect.height };
}

interface ObserverAdapter<Entry> {
  readonly isSupported: boolean;
  readonly observe: (target: Element) => void;
  readonly unobserve: (target: Element) => void;
  readonly translate: (entries: readonly Entry[]) => void;
  readonly release: () => void;
}

/** Shared observe/unobserve/disconnect/dispose lifecycle for both wrappers. */
function createObservationLifecycle<Entry>(adapter: ObserverAdapter<Entry>) {
  const observed = new Set<Element>();
  const observedCount = shallowRef(0);
  let disposed = false;

  const assertUsable = (): void => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
  };

  const disconnect = (): void => {
    assertUsable();
    for (const target of observed) adapter.unobserve(target);
    observed.clear();
    observedCount.value = 0;
  };

  return {
    isSupported: adapter.isSupported,
    observedCount: shallowReadonly(observedCount),
    isObserved: (target: Element) => observed.has(target),
    translate: (entries: readonly Entry[]) => adapter.translate(entries),
    observe: (target: Element): void => {
      assertUsable();
      assertElement(target);
      if (!adapter.isSupported || observed.has(target)) return;
      adapter.observe(target);
      observed.add(target);
      observedCount.value = observed.size;
    },
    unobserve: (target: Element): void => {
      assertUsable();
      if (!observed.delete(target)) return;
      adapter.unobserve(target);
      observedCount.value = observed.size;
    },
    disconnect,
    dispose: (): void => {
      if (disposed) return;
      disconnect();
      disposed = true;
      adapter.release();
    },
  };
}

/** Create an SSR-safe `ResizeObserver` wrapper. Observation no-ops without platform support. */
export function createSizeObserver(options: SizeObserverOptions): SizeObserverController {
  assertCallback("onResize", options.onResize);
  const box = options.box ?? "border-box";
  if (!boxes.has(box)) {
    throw new TypeError(`${invalidOptionDiagnostic}: box must be border-box or content-box`);
  }

  const Observer = typeof globalThis.ResizeObserver === "function" ? ResizeObserver : null;
  const isSupported = Observer !== null;
  let observer: ResizeObserver | null = null;

  const lifecycle = createObservationLifecycle<ResizeObserverEntry>({
    isSupported,
    observe(target) {
      if (!Observer) return;
      observer ??= new Observer((entries) => lifecycle.translate(entries));
      observer.observe(target, { box });
    },
    unobserve(target) {
      observer?.unobserve(target);
    },
    translate(entries) {
      const changes: SizeObserverEntry[] = [];
      for (const entry of entries) {
        if (!lifecycle.isObserved(entry.target)) continue;
        changes.push(Object.freeze({ target: entry.target, ...readBoxSize(entry, box) }));
      }
      if (changes.length > 0) options.onResize(Object.freeze(changes));
    },
    release() {
      observer?.disconnect();
      observer = null;
    },
  });

  return Object.freeze({
    isSupported,
    observedCount: lifecycle.observedCount,
    observe: lifecycle.observe,
    unobserve: lifecycle.unobserve,
    disconnect: lifecycle.disconnect,
    dispose: lifecycle.dispose,
  });
}

/** Create an SSR-safe `IntersectionObserver` wrapper. Observation no-ops without support. */
export function createVisibilityObserver(
  options: VisibilityObserverOptions,
): VisibilityObserverController {
  assertCallback("onVisibilityChange", options.onVisibilityChange);

  const Observer =
    typeof globalThis.IntersectionObserver === "function" ? IntersectionObserver : null;
  const isSupported = Observer !== null;
  let observer: IntersectionObserver | null = null;

  const lifecycle = createObservationLifecycle<IntersectionObserverEntry>({
    isSupported,
    observe(target) {
      if (!Observer) return;
      observer ??= new Observer((entries) => lifecycle.translate(entries), {
        root: options.root ?? null,
        rootMargin: options.rootMargin ?? "0px",
        threshold: (options.threshold ?? 0) as number | number[],
      });
      observer.observe(target);
    },
    unobserve(target) {
      observer?.unobserve(target);
    },
    translate(entries) {
      const changes: VisibilityObserverEntry[] = [];
      for (const entry of entries) {
        if (!lifecycle.isObserved(entry.target)) continue;
        changes.push(
          Object.freeze({
            target: entry.target,
            isIntersecting: entry.isIntersecting,
            intersectionRatio: entry.intersectionRatio,
          }),
        );
      }
      if (changes.length > 0) options.onVisibilityChange(Object.freeze(changes));
    },
    release() {
      observer?.disconnect();
      observer = null;
    },
  });

  return Object.freeze({
    isSupported,
    observedCount: lifecycle.observedCount,
    observe: lifecycle.observe,
    unobserve: lifecycle.unobserve,
    disconnect: lifecycle.disconnect,
    dispose: lifecycle.dispose,
  });
}

function bindToScope<Controller extends { readonly dispose: () => void }>(
  controller: Controller,
): Controller {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  onScopeDispose(controller.dispose);
  return controller;
}

/** Create a size observer disposed with the current Vue effect scope. */
export function useSizeObserver(options: SizeObserverOptions): SizeObserverController {
  return bindToScope(createSizeObserver(options));
}

/** Create a visibility observer disposed with the current Vue effect scope. */
export function useVisibilityObserver(
  options: VisibilityObserverOptions,
): VisibilityObserverController {
  return bindToScope(createVisibilityObserver(options));
}
