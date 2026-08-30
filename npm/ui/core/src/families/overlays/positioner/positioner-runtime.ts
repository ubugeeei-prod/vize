import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef, toValue, watch } from "vue";
import type { MaybeRefOrGetter } from "vue";

import { computePosition, readRect } from "./positioner-geometry.ts";
import { arrowStyle, computeAvailableSize, hostStyle, sizeStyle } from "./positioner-size.ts";
import {
  insetViewport,
  ownerDocumentOf,
  readSafeAreaInsets,
  visualViewportRect,
  zeroSafeAreaInsets,
} from "./positioner-viewport.ts";
import type {
  Placement,
  PositionerArrowStyle,
  PositionerController,
  PositionerElement,
  PositionerOptions,
  PositionerStrategy,
  PositionerStyle,
  Rect,
  SafeAreaInsets,
} from "./positioner-types.ts";

const invalidOptionDiagnostic = "VIZE_UI_POSITIONER_OPTION";
const disposedDiagnostic = "VIZE_UI_POSITIONER_DISPOSED";
const setupDiagnostic = "VIZE_UI_POSITIONER_SETUP";
const placements = new Set<Placement>([
  "bottom",
  "bottom-end",
  "bottom-start",
  "left",
  "left-end",
  "left-start",
  "right",
  "right-end",
  "right-start",
  "top",
  "top-end",
  "top-start",
]);
const strategies = new Set<PositionerStrategy>(["absolute", "fixed"]);

function readBoolean(
  value: MaybeRefOrGetter<boolean | undefined> | undefined,
  fallback: boolean,
): boolean {
  const resolved = toValue(value);
  if (resolved === undefined) return fallback;
  if (typeof resolved !== "boolean") {
    throw new TypeError(`${invalidOptionDiagnostic}: expected a boolean`);
  }
  return resolved;
}

function readNumber(
  value: MaybeRefOrGetter<number | undefined> | undefined,
  fallback: number,
): number {
  const resolved = toValue(value);
  if (resolved === undefined) return fallback;
  if (typeof resolved !== "number" || !Number.isFinite(resolved)) {
    throw new TypeError(`${invalidOptionDiagnostic}: expected a finite number`);
  }
  return resolved;
}

function readPlacement(value: PositionerOptions["placement"]): Placement {
  const resolved = toValue(value);
  if (resolved === undefined) return "bottom";
  if (!placements.has(resolved)) {
    throw new TypeError(`${invalidOptionDiagnostic}: placement must be a named side or alignment`);
  }
  return resolved;
}

function readStrategy(value: PositionerOptions["strategy"]): PositionerStrategy {
  const resolved = toValue(value);
  if (resolved === undefined) return "fixed";
  if (!strategies.has(resolved)) {
    throw new TypeError(`${invalidOptionDiagnostic}: strategy must resolve to absolute or fixed`);
  }
  return resolved;
}

function readDirection(value: PositionerOptions["direction"]): "ltr" | "rtl" {
  const resolved = toValue(value);
  if (resolved === undefined) return "ltr";
  if (resolved !== "ltr" && resolved !== "rtl") {
    throw new TypeError(`${invalidOptionDiagnostic}: direction must resolve to ltr or rtl`);
  }
  return resolved;
}

function readViewport(value: PositionerOptions["viewport"]): Rect | null {
  const resolved = toValue(value);
  if (resolved === undefined) return null;
  if (
    typeof resolved !== "object" ||
    resolved === null ||
    typeof resolved.x !== "number" ||
    typeof resolved.y !== "number" ||
    typeof resolved.width !== "number" ||
    typeof resolved.height !== "number"
  ) {
    throw new TypeError(`${invalidOptionDiagnostic}: viewport must resolve to a box`);
  }
  return resolved;
}

function validateOptions(options: PositionerOptions): void {
  if (typeof options.placement !== "function") readPlacement(options.placement);
  if (typeof options.strategy !== "function") readStrategy(options.strategy);
  if (typeof options.direction !== "function") readDirection(options.direction);
  if (typeof options.offset !== "function") readNumber(options.offset, 0);
  if (typeof options.collisionPadding !== "function") readNumber(options.collisionPadding, 0);
  if (typeof options.arrowPadding !== "function") readNumber(options.arrowPadding, 0);
  if (typeof options.flip !== "function") readBoolean(options.flip, true);
  if (typeof options.shift !== "function") readBoolean(options.shift, true);
  if (typeof options.size !== "function") readBoolean(options.size, false);
  if (typeof options.safeArea !== "function") readBoolean(options.safeArea, false);
  if (typeof options.hide !== "function") readBoolean(options.hide, true);
  if (typeof options.updateOnScroll !== "function") readBoolean(options.updateOnScroll, true);
  if (typeof options.updateOnResize !== "function") readBoolean(options.updateOnResize, true);
  if (typeof options.viewport !== "function") readViewport(options.viewport);
}

function measure(element: PositionerElement | null): Rect | null {
  if (element === null) return null;
  return readRect(element.getBoundingClientRect());
}

function floatingOffsetParent(element: PositionerElement | null): Element | null {
  if (element === null || !("offsetParent" in element)) return null;
  const parent = (element as HTMLElement).offsetParent;
  return parent instanceof Element ? parent : null;
}

/** Create an SSR-safe floating placement controller. */
export function createPositioner(options: PositionerOptions = {}): PositionerController {
  validateOptions(options);
  const x = shallowRef(0);
  const y = shallowRef(0);
  const arrowX = shallowRef<number | null>(null);
  const arrowY = shallowRef<number | null>(null);
  const availableWidth = shallowRef<number | null>(null);
  const availableHeight = shallowRef<number | null>(null);
  const hidden = shallowRef(false);
  const ready = shallowRef(false);
  const resolvedPlacement = shallowRef<Placement>(readPlacement(options.placement));
  const strategy = shallowRef<PositionerStrategy>(readStrategy(options.strategy));
  const style = shallowRef<PositionerStyle>(hostStyle(strategy.value, 0, 0));
  const arrowStyles = shallowRef<PositionerArrowStyle>(arrowStyle(null, null));
  let disposed = false;
  let reference: PositionerElement | null = null;
  let floating: PositionerElement | null = null;
  let arrow: PositionerElement | null = null;
  let safeAreaInsets: SafeAreaInsets = zeroSafeAreaInsets;
  const listeners: Array<() => void> = [];

  const assertAlive = (): void => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
  };

  // Safe-area insets change with orientation and zoom, not with scrolling,
  // so the env() probe runs on attach, resize, and option changes only.
  const refreshSafeArea = (): void => {
    safeAreaInsets = readBoolean(options.safeArea, false)
      ? readSafeAreaInsets(ownerDocumentOf(floating))
      : zeroSafeAreaInsets;
  };

  const detach = (): void => {
    while (listeners.length > 0) listeners.pop()?.();
  };

  const attach = (): void => {
    detach();
    refreshSafeArea();
    if (typeof globalThis.addEventListener !== "function") return;
    const onScroll = () => {
      if (readBoolean(options.updateOnScroll, true)) update();
    };
    const onResize = () => {
      refreshSafeArea();
      if (readBoolean(options.updateOnResize, true)) update();
    };
    globalThis.addEventListener("scroll", onScroll, true);
    listeners.push(() => globalThis.removeEventListener("scroll", onScroll, true));
    globalThis.addEventListener("resize", onResize);
    listeners.push(() => globalThis.removeEventListener("resize", onResize));
    const visual = globalThis.visualViewport;
    if (visual) {
      visual.addEventListener("resize", onResize);
      visual.addEventListener("scroll", onScroll);
      listeners.push(() => {
        visual.removeEventListener("resize", onResize);
        visual.removeEventListener("scroll", onScroll);
      });
    }
    if (typeof ResizeObserver === "function") {
      const observer = new ResizeObserver(onResize);
      if (reference instanceof Element) observer.observe(reference);
      if (floating instanceof Element) observer.observe(floating);
      listeners.push(() => observer.disconnect());
    }
  };

  const update = (): void => {
    assertAlive();
    const referenceRect = measure(reference);
    const floatingRect = measure(floating);
    if (referenceRect === null || floatingRect === null) return;

    const nextStrategy = readStrategy(options.strategy);
    const collisionPadding = readNumber(options.collisionPadding, 0);
    const offset = readNumber(options.offset, 0);
    const viewport = insetViewport(
      readViewport(options.viewport) ?? visualViewportRect(),
      safeAreaInsets,
    );
    const result = computePosition({
      arrow: measure(arrow),
      arrowPadding: readNumber(options.arrowPadding, 0),
      collisionPadding,
      flip: readBoolean(options.flip, true),
      floating: floatingRect,
      hide: readBoolean(options.hide, true),
      offset,
      placement: readPlacement(options.placement),
      reference: referenceRect,
      rtl: readDirection(options.direction) === "rtl",
      shift: readBoolean(options.shift, true),
      viewport,
    });
    const available = computeAvailableSize({
      collisionPadding,
      offset,
      placement: result.placement,
      reference: referenceRect,
      viewport,
    });

    let nextX = result.x;
    let nextY = result.y;
    if (nextStrategy === "absolute") {
      const parent = measure(floatingOffsetParent(floating));
      if (parent !== null) {
        nextX -= parent.x;
        nextY -= parent.y;
      }
    }

    x.value = nextX;
    y.value = nextY;
    arrowX.value = result.arrowX;
    arrowY.value = result.arrowY;
    availableWidth.value = available.width;
    availableHeight.value = available.height;
    hidden.value = result.hidden;
    resolvedPlacement.value = result.placement;
    strategy.value = nextStrategy;
    style.value =
      hostStyle(nextStrategy, nextX, nextY) +
      (readBoolean(options.size, false) ? sizeStyle(available) : "");
    arrowStyles.value = arrowStyle(result.arrowX, result.arrowY);
    ready.value = true;
  };

  const stopWatch = watch(
    () => [
      toValue(options.placement),
      toValue(options.strategy),
      toValue(options.direction),
      toValue(options.offset),
      toValue(options.collisionPadding),
      toValue(options.arrowPadding),
      toValue(options.flip),
      toValue(options.shift),
      toValue(options.size),
      toValue(options.safeArea),
      toValue(options.hide),
      toValue(options.viewport),
    ],
    () => {
      if (disposed) return;
      refreshSafeArea();
      if (ready.value) update();
    },
    { flush: "sync" },
  );

  return Object.freeze({
    arrowX: shallowReadonly(arrowX),
    arrowY: shallowReadonly(arrowY),
    arrowStyle: shallowReadonly(arrowStyles),
    availableHeight: shallowReadonly(availableHeight),
    availableWidth: shallowReadonly(availableWidth),
    dispose: () => {
      if (disposed) return;
      disposed = true;
      detach();
      stopWatch();
    },
    hidden: shallowReadonly(hidden),
    ready: shallowReadonly(ready),
    resolvedPlacement: shallowReadonly(resolvedPlacement),
    setArrow: (element: PositionerElement | null) => {
      assertAlive();
      if (arrow === element) return;
      arrow = element;
      if (ready.value) update();
    },
    setFloating: (element: PositionerElement | null) => {
      assertAlive();
      if (floating === element) return;
      floating = element;
      attach();
      update();
    },
    setReference: (element: PositionerElement | null) => {
      assertAlive();
      if (reference === element) return;
      reference = element;
      attach();
      update();
    },
    strategy: shallowReadonly(strategy),
    style: shallowReadonly(style),
    update,
    x: shallowReadonly(x),
    y: shallowReadonly(y),
  });
}

/** Create a positioner disposed with the current Vue effect scope. */
export function usePositioner(options: PositionerOptions = {}): PositionerController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createPositioner(options);
  onScopeDispose(controller.dispose);
  return controller;
}
