import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef, watch } from "vue";

import { createSizeObserver } from "./measure-runtime.ts";
import { createMeasureCache } from "./virtualizer-measure-cache.ts";
import {
  disposedDiagnostic,
  invalidOptionDiagnostic,
  resolveVirtualizerOptions,
  setupDiagnostic,
} from "./virtualizer-options.ts";
import type {
  VirtualItem,
  VirtualizerAlignment,
  VirtualizerController,
  VirtualizerOptions,
  VirtualizerRect,
  VirtualRange,
} from "./virtualizer-types.ts";
import { createViewportBinding } from "./virtualizer-viewport.ts";
import { computeVirtualWindow } from "./virtualizer-window.ts";

const alignments = new Set<VirtualizerAlignment>(["auto", "center", "end", "start"]);

function rangesEqual(left: VirtualRange | null, right: VirtualRange | null): boolean {
  if (left === right) return true;
  return (
    left !== null &&
    right !== null &&
    left.startIndex === right.startIndex &&
    left.endIndex === right.endIndex
  );
}

/** Create an SSR-safe windowing controller for one scroll axis. */
export function createVirtualizer(options: VirtualizerOptions): VirtualizerController {
  const readers = resolveVirtualizerOptions(options);
  let disposed = false;

  const scrollOffset = shallowRef(readers.initialScrollOffset);
  const viewportRect = shallowRef<VirtualizerRect>(readers.initialRect);
  const mainAxisSize = (): number =>
    readers.readOrientation() === "vertical" ? viewportRect.value.height : viewportRect.value.width;
  const viewportSize = shallowRef(mainAxisSize());
  const layoutVersion = shallowRef(0);
  const bump = (): void => {
    layoutVersion.value++;
  };

  const cache = createMeasureCache({
    getCount: readers.readCount,
    getLanes: readers.readLanes,
    getGap: readers.readGap,
    getPaddingStart: () => readers.paddingStart,
    resolveBaseSize: readers.resolveBaseSize,
    usesExactSizes: readers.usesExactSizes,
  });

  const virtualItems = shallowRef<readonly VirtualItem[]>(Object.freeze([]));
  const range = shallowRef<VirtualRange | null>(null);
  const totalSize = shallowRef(0);
  const activeStickyIndex = shallowRef<number | null>(null);

  const assertUsable = (): void => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
  };

  const assertIndex = (index: number): void => {
    const count = readers.readCount();
    if (!Number.isInteger(index) || index < 0 || index >= count) {
      throw new RangeError(
        `${invalidOptionDiagnostic}: index ${index} is outside the collection of ${count}`,
      );
    }
  };

  const recompute = (): void => {
    const window = computeVirtualWindow({
      cache,
      count: readers.readCount(),
      lanes: readers.readLanes(),
      overscan: readers.readOverscan(),
      scrollOffset: scrollOffset.value,
      viewportSize: viewportSize.value,
      stickyIndexes: readers.readStickyIndexes(),
      getItemKey: readers.getItemKey,
    });
    virtualItems.value = window.items;
    totalSize.value = cache.contentEnd() + readers.paddingEnd;
    activeStickyIndex.value = window.activeStickyIndex;
    if (!rangesEqual(range.value, window.range)) {
      range.value = window.range;
      options.onRangeChange?.(window.range);
    }
  };

  const binding = createViewportBinding({
    getOrientation: readers.readOrientation,
    onScrollOffset(offset) {
      if (scrollOffset.value !== offset) scrollOffset.value = offset;
    },
    onViewportSize(rect) {
      viewportRect.value = rect;
      const next = mainAxisSize();
      if (viewportSize.value !== next) viewportSize.value = next;
    },
  });

  const anchorAdjust = (index: number, delta: number): void => {
    if (!readers.anchorScroll || delta === 0) return;
    if (cache.placement(index).start >= scrollOffset.value) return;
    const next = Math.max(0, scrollOffset.value + delta);
    scrollOffset.value = next;
    binding.applyOffset(next);
  };

  const applyMeasured = (index: number, size: number): void => {
    if (readers.usesExactSizes()) return;
    const wasMeasured = cache.measuredSize(index) !== undefined;
    const delta = cache.setMeasured(index, size);
    if (delta === 0 && wasMeasured) return;
    anchorAdjust(index, delta);
    bump();
  };

  const indexElements = new Map<number, Element>();
  const elementIndexes = new Map<Element, number>();

  function forget(element: Element): void {
    measureObserver.unobserve(element);
    const index = elementIndexes.get(element);
    elementIndexes.delete(element);
    if (index !== undefined && indexElements.get(index) === element) indexElements.delete(index);
  }

  const measureObserver = createSizeObserver({
    onResize(entries) {
      for (const entry of entries) {
        const index = elementIndexes.get(entry.target);
        if (index === undefined) continue;
        if (!entry.target.isConnected) {
          // Disconnected-node recovery: release the node, keep its measurement.
          forget(entry.target);
          continue;
        }
        applyMeasured(index, readers.readOrientation() === "vertical" ? entry.height : entry.width);
      }
    },
  });

  const scrollToOffset = (offset: number): void => {
    assertUsable();
    if (typeof offset !== "number" || !Number.isFinite(offset)) {
      throw new TypeError(`${invalidOptionDiagnostic}: offset must be a finite number`);
    }
    const limit = Math.max(0, cache.contentEnd() + readers.paddingEnd - viewportSize.value);
    const clamped = Math.min(Math.max(0, offset), limit);
    if (scrollOffset.value !== clamped) scrollOffset.value = clamped;
    binding.applyOffset(clamped);
  };

  const stopStructureWatch = watch(
    () =>
      [
        readers.readCount(),
        readers.readLanes(),
        readers.readGap(),
        readers.readOrientation(),
      ] as const,
    (next, previous) => {
      if (next[0] !== previous[0] || next[1] !== previous[1] || next[2] !== previous[2]) {
        cache.invalidateFrom(0);
      }
      if (next[3] !== previous[3]) viewportSize.value = mainAxisSize();
      recompute();
    },
    { flush: "sync" },
  );

  const stopWindowWatch = watch(
    () =>
      [
        scrollOffset.value,
        viewportSize.value,
        layoutVersion.value,
        readers.readOverscan(),
        readers.readStickyIndexes(),
      ] as const,
    recompute,
    { flush: "sync" },
  );

  recompute();

  const controller: VirtualizerController = Object.freeze<VirtualizerController>({
    virtualItems: shallowReadonly(virtualItems),
    range: shallowReadonly(range),
    totalSize: shallowReadonly(totalSize),
    scrollOffset: shallowReadonly(scrollOffset),
    viewportSize: shallowReadonly(viewportSize),
    activeStickyIndex: shallowReadonly(activeStickyIndex),
    setViewport(element) {
      assertUsable();
      if (element !== null && element.nodeType !== 1) {
        throw new TypeError(`${invalidOptionDiagnostic}: the viewport must be an element`);
      }
      binding.attach(element);
    },
    measureElement(element, index) {
      assertUsable();
      if (element === null) {
        const existing = indexElements.get(index);
        if (existing) forget(existing);
        return;
      }
      if (element.nodeType !== 1) {
        throw new TypeError(`${invalidOptionDiagnostic}: measured targets must be elements`);
      }
      assertIndex(index);
      const previous = indexElements.get(index);
      if (previous && previous !== element) forget(previous);
      const previousIndex = elementIndexes.get(element);
      if (previousIndex !== undefined && previousIndex !== index) {
        indexElements.delete(previousIndex);
      }
      indexElements.set(index, element);
      elementIndexes.set(element, index);
      measureObserver.observe(element);
      const rect =
        typeof element.getBoundingClientRect === "function"
          ? element.getBoundingClientRect()
          : null;
      const extent =
        rect === null ? 0 : readers.readOrientation() === "vertical" ? rect.height : rect.width;
      if (extent > 0) applyMeasured(index, extent);
    },
    resizeItem(index, size) {
      assertUsable();
      assertIndex(index);
      applyMeasured(index, size);
    },
    invalidateMeasurements(fromIndex = 0) {
      assertUsable();
      if (!Number.isInteger(fromIndex) || fromIndex < 0) {
        throw new TypeError(`${invalidOptionDiagnostic}: fromIndex must be a non-negative integer`);
      }
      cache.clearMeasuredFrom(fromIndex);
      bump();
    },
    notifyPrepended(prepended) {
      assertUsable();
      if (!Number.isInteger(prepended) || prepended <= 0) {
        throw new TypeError(
          `${invalidOptionDiagnostic}: notifyPrepended requires a positive integer`,
        );
      }
      cache.shiftMeasurements(prepended);
      const shiftedIndexElements = [...indexElements.entries()];
      indexElements.clear();
      for (const [index, element] of shiftedIndexElements) {
        indexElements.set(index + prepended, element);
        elementIndexes.set(element, index + prepended);
      }
      if (readers.anchorScroll) {
        let delta = 0;
        const gap = readers.readGap();
        const lanes = readers.readLanes();
        for (let index = 0; index < prepended; index++) {
          delta += readers.resolveBaseSize(index) + gap;
        }
        bump();
        scrollToOffset(scrollOffset.value + delta / lanes);
      } else {
        bump();
      }
    },
    scrollToOffset,
    scrollToIndex(index, alignment = "auto") {
      assertUsable();
      assertIndex(index);
      if (!alignments.has(alignment)) {
        throw new TypeError(`${invalidOptionDiagnostic}: unknown alignment ${String(alignment)}`);
      }
      const placement = cache.placement(index);
      const viewport = viewportSize.value;
      let resolved = alignment;
      if (resolved === "auto") {
        if (placement.start < scrollOffset.value) resolved = "start";
        else if (placement.end > scrollOffset.value + viewport) resolved = "end";
        else return;
      }
      if (resolved === "start") scrollToOffset(placement.start);
      else if (resolved === "end") scrollToOffset(placement.end - viewport);
      else scrollToOffset(placement.start - (viewport - placement.size) / 2);
    },
    createScrollSnapshot() {
      assertUsable();
      const anchorIndex = range.value?.startIndex ?? null;
      return Object.freeze({
        offset: scrollOffset.value,
        anchorIndex,
        anchorGap:
          anchorIndex === null ? 0 : scrollOffset.value - cache.placement(anchorIndex).start,
      });
    },
    restoreScroll(snapshot) {
      assertUsable();
      const { anchorIndex } = snapshot;
      if (
        anchorIndex !== null &&
        Number.isInteger(anchorIndex) &&
        anchorIndex >= 0 &&
        anchorIndex < readers.readCount() &&
        Number.isFinite(snapshot.anchorGap)
      ) {
        scrollToOffset(cache.placement(anchorIndex).start + snapshot.anchorGap);
        return;
      }
      scrollToOffset(snapshot.offset);
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      stopStructureWatch();
      stopWindowWatch();
      binding.dispose();
      measureObserver.dispose();
      indexElements.clear();
      elementIndexes.clear();
    },
  });

  return controller;
}

/** Create a virtualizer disposed with the current Vue effect scope. */
export function useVirtualizer(options: VirtualizerOptions): VirtualizerController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createVirtualizer(options);
  onScopeDispose(controller.dispose);
  return controller;
}
