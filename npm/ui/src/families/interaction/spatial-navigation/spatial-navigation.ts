import { getCurrentScope, onScopeDispose } from "vue";

import {
  capture,
  isEditableDescendant,
  keyDirection,
  normalizeRect,
  rankItem,
  readAlgorithm,
  readBoolean,
  readBoundary,
  readDirection,
  readFocus,
  selectCandidate,
  selectWrappedCandidate,
  surfaceErrors,
  validateOptions,
} from "./spatial-navigation-internal.ts";
import type { RankedSpatialItem } from "./spatial-navigation-internal.ts";
import type {
  SpatialNavigationAlgorithm,
  SpatialNavigationBoundary,
  SpatialNavigationChange,
  SpatialNavigationController,
  SpatialNavigationDirection,
  SpatialNavigationOptions,
  SpatialNavigationProps,
  SpatialNavigationRect,
} from "./spatial-navigation-types.ts";
import type { CollectionItem, CollectionKey } from "../../foundations/collection/collection.ts";

const disposedDiagnostic = "VIZE_UI_SPATIAL_NAVIGATION_DISPOSED";
const originDiagnostic = "VIZE_UI_SPATIAL_NAVIGATION_ORIGIN";
const setupDiagnostic = "VIZE_UI_SPATIAL_NAVIGATION_SETUP";

interface Resolution<Key extends CollectionKey, Value> {
  readonly algorithm: SpatialNavigationAlgorithm;
  readonly originKey: Key;
  readonly target: RankedSpatialItem<Key, Value> | null;
}

/** Create a geometry-driven, SSR-safe arrow navigation controller. */
export function createSpatialNavigation<Key extends CollectionKey, Value>(
  options: SpatialNavigationOptions<Key, Value>,
): SpatialNavigationController<Key> {
  validateOptions(options);
  const registry = options.registry;
  let disposed = false;

  const assertActive = () => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
  };
  const rectFor = (item: CollectionItem<Key, Value>): SpatialNavigationRect | null => {
    const supplied = options.getRect?.(item);
    if (supplied !== undefined && supplied !== null) return normalizeRect(supplied);
    if (options.getRect) return null;
    const read = (item.element as Partial<Element> | null)?.getBoundingClientRect;
    return typeof read === "function" ? normalizeRect(read.call(item.element)) : null;
  };
  const originItem = (fromKey?: Key): CollectionItem<Key, Value> | null => {
    const items = registry.navigableItems.value;
    if (items.length === 0) return null;
    const key = fromKey ?? registry.activeKey.value ?? items[0]?.key;
    const item = items.find((candidate) => candidate.key === key);
    if (!item) {
      throw new Error(`${originDiagnostic}: ${String(key)} is not a navigable collection item`);
    }
    return item;
  };
  const resolve = (
    directionInput: SpatialNavigationDirection,
    fromKey?: Key,
  ): Resolution<Key, Value> | null => {
    assertActive();
    const direction = readDirection(directionInput);
    if (readBoolean(options.isDisabled, "isDisabled")) return null;
    const origin = originItem(fromKey);
    if (!origin) return null;
    const originRect = rectFor(origin);
    if (!originRect) {
      throw new Error(`${originDiagnostic}: ${String(origin.key)} has no measurable rectangle`);
    }
    const measured: Array<{
      item: CollectionItem<Key, Value>;
      rect: SpatialNavigationRect;
      index: number;
    }> = [];
    const candidates: Array<RankedSpatialItem<Key, Value>> = [];
    for (const [index, item] of registry.navigableItems.value.entries()) {
      if (item.key === origin.key) continue;
      const rect = rectFor(item);
      if (!rect) continue;
      measured.push({ item, rect, index });
      const candidate = rankItem(item, rect, index, originRect, direction);
      if (candidate) candidates.push(candidate);
    }
    const algorithm = readAlgorithm(options.algorithm);
    let target = selectCandidate(candidates, algorithm) ?? null;
    if (!target && readBoolean(options.loop, "loop")) {
      target = selectWrappedCandidate(measured, originRect, direction) ?? null;
    }
    return { algorithm, originKey: origin.key, target };
  };

  const synchronizeDom = (
    item: CollectionItem<Key, Value>,
    originalEvent: Event | null,
    errors: unknown[],
  ): void => {
    const focusBehavior = readFocus(options.focusBehavior);
    const preventScroll = readBoolean(options.preventScroll, "preventScroll");
    if (focusBehavior === "focus") {
      const focus = (item.element as Partial<HTMLElement> | null)?.focus;
      if (typeof focus !== "function") {
        errors.push(new Error("VIZE_UI_SPATIAL_NAVIGATION_FOCUS: target item is not focusable"));
      } else {
        capture(errors, () => {
          try {
            focus.call(item.element, { preventScroll });
          } catch {
            focus.call(item.element);
          }
        });
      }
    }
    const shouldReveal = options.scrollIntoView || focusBehavior === "logical" || preventScroll;
    if (!shouldReveal) return;
    if (options.scrollIntoView) {
      capture(errors, () => options.scrollIntoView?.(item, originalEvent));
      return;
    }
    const reveal = (item.element as Partial<HTMLElement> | null)?.scrollIntoView;
    if (typeof reveal === "function") {
      capture(errors, () => reveal.call(item.element, { block: "nearest", inline: "nearest" }));
    }
  };
  const transition = (
    direction: SpatialNavigationDirection,
    resolution: Resolution<Key, Value> | null,
    originalEvent: Event | null,
  ): Key | null => {
    const errors: unknown[] = [];
    if (!resolution) return null;
    if (!resolution.target) {
      const boundary: SpatialNavigationBoundary<Key> = Object.freeze({
        direction,
        key: resolution.originKey,
        originalEvent,
      });
      capture(errors, () => options.onBoundary?.(boundary));
      surfaceErrors(errors, "Spatial navigation boundary callback failed");
      return null;
    }
    const { item, score } = resolution.target;
    const previousKey = registry.activeKey.value;
    capture(errors, () => {
      registry.setActiveKey(item.key);
    });
    if (registry.activeKey.value === item.key && item.key !== previousKey) {
      capture(errors, () => synchronizeDom(item, originalEvent, errors));
      const change: SpatialNavigationChange<Key> = Object.freeze({
        algorithm: resolution.algorithm,
        direction,
        key: item.key,
        previousKey,
        originalEvent,
        score,
      });
      capture(errors, () => options.onNavigate?.(change));
    }
    surfaceErrors(errors, "Spatial navigation transition failed");
    return registry.activeKey.value;
  };

  const navigate = (
    directionInput: SpatialNavigationDirection,
    originalEvent: Event | null = null,
  ): Key | null => {
    const direction = readDirection(directionInput);
    return transition(direction, resolve(direction), originalEvent);
  };
  const spatialNavigationProps: Readonly<SpatialNavigationProps> = Object.freeze({
    onKeydown(event: KeyboardEvent) {
      if (event.defaultPrevented || isEditableDescendant(event)) return;
      const direction = keyDirection(event);
      if (!direction || readBoolean(options.isDisabled, "isDisabled")) return;
      const resolution = resolve(direction);
      if (
        resolution &&
        (resolution.target || readBoundary(options.boundaryBehavior) === "contain")
      ) {
        event.preventDefault();
      }
      transition(direction, resolution, event);
    },
  });

  return Object.freeze({
    spatialNavigationProps,
    findTarget: (direction: SpatialNavigationDirection, fromKey?: Key) =>
      resolve(direction, fromKey)?.target?.item.key ?? null,
    navigate,
    dispose: () => {
      disposed = true;
    },
  });
}

/** Create a spatial controller disposed with the current Vue effect scope. */
export function useSpatialNavigation<Key extends CollectionKey, Value>(
  options: SpatialNavigationOptions<Key, Value>,
): SpatialNavigationController<Key> {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createSpatialNavigation(options);
  onScopeDispose(controller.dispose);
  return controller;
}

export type {
  SpatialNavigationAlgorithm,
  SpatialNavigationBoundary,
  SpatialNavigationBoundaryBehavior,
  SpatialNavigationChange,
  SpatialNavigationController,
  SpatialNavigationDirection,
  SpatialNavigationFocusBehavior,
  SpatialNavigationOptions,
  SpatialNavigationProps,
  SpatialNavigationRect,
} from "./spatial-navigation-types.ts";
