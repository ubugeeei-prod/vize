import { computed } from "vue";
import type { ComputedRef } from "vue";

import type { ChartContextValue } from "./chart-context.ts";

/** Geometry and description of one navigable data point. */
export interface ChartNavigablePoint {
  readonly x: number;
  readonly y: number;
  readonly label: string;
}

/** Props bound to each navigable mark. */
export interface ChartNavigationItemProps {
  readonly tabindex: 0 | -1;
  readonly onFocus: () => void;
  readonly onPointerenter: (event: PointerEvent) => void;
  readonly onPointerleave: (event: PointerEvent) => void;
}

/** Roving-focus navigation over the marks of one series. */
export interface ChartNavigation {
  readonly activeIndex: ComputedRef<number | null>;
  readonly itemProps: (index: number) => ChartNavigationItemProps;
  readonly registerElement: (index: number, element: unknown) => void;
  readonly onKeydown: (event: KeyboardEvent) => void;
  readonly activateNearest: (event: MouseEvent) => void;
  readonly deactivate: () => void;
}

/**
 * Roving-focus keyboard navigation over the data points of one series.
 *
 * One mark is in the tab order at a time. Arrow keys move between points in
 * data order (Right/Down forward, Left/Up back), Home/End jump to the ends and
 * Escape clears the active point. Keyboard moves announce the point's label
 * through the chart's live region; pointer hover only updates the active
 * point, so tooltips and crosshairs follow both inputs.
 */
export function useChartNavigation(options: {
  readonly context: ChartContextValue;
  readonly series: () => string;
  readonly count: () => number;
  readonly point: (index: number) => ChartNavigablePoint | null;
  readonly disabled: () => boolean;
}): ChartNavigation {
  const { context } = options;
  const elements = new Map<number, Element>();
  const activeIndex = computed(() => {
    const active = context.active.value;
    return active !== null && active.series === options.series() ? active.index : null;
  });
  const tabStop = computed(() => {
    const index = activeIndex.value;
    return index !== null && index < options.count() ? index : 0;
  });

  const activate = (index: number, reason: "keyboard" | "pointer", focus: boolean) => {
    const point = options.point(index);
    if (point === null || options.disabled()) return;
    context.setActive(
      { series: options.series(), index, x: point.x, y: point.y, label: point.label },
      reason,
    );
    if (reason === "keyboard") context.announce(point.label);
    if (focus) {
      const element = elements.get(index);
      if (element instanceof SVGElement || element instanceof HTMLElement) element.focus();
    }
  };

  const onKeydown = (event: KeyboardEvent) => {
    const count = options.count();
    if (count === 0 || options.disabled()) return;
    const current = activeIndex.value ?? tabStop.value;
    let next: number | null = null;
    if (event.key === "ArrowRight" || event.key === "ArrowDown")
      next = Math.min(count - 1, current + 1);
    else if (event.key === "ArrowLeft" || event.key === "ArrowUp") next = Math.max(0, current - 1);
    else if (event.key === "Home") next = 0;
    else if (event.key === "End") next = count - 1;
    else if (event.key === "Escape") {
      if (activeIndex.value === null) return;
      context.setActive(null, "keyboard");
      event.preventDefault();
      return;
    }
    if (next === null) return;
    event.preventDefault();
    activate(next, "keyboard", true);
  };

  const activateNearest = (event: MouseEvent) => {
    const position = context.plotPoint(event);
    if (position === null) return;
    let best: number | null = null;
    let distance = Infinity;
    for (let index = 0; index < options.count(); index++) {
      const point = options.point(index);
      if (point === null) continue;
      const delta = Math.abs(point.x - position.x);
      if (delta < distance) {
        distance = delta;
        best = index;
      }
    }
    if (best !== null) activate(best, "pointer", false);
  };

  return {
    activeIndex,
    itemProps: (index) => ({
      tabindex: index === tabStop.value && !options.disabled() ? 0 : -1,
      onFocus: () => {
        if (activeIndex.value !== index) activate(index, "keyboard", false);
      },
      onPointerenter: (event) => {
        if (event.pointerType !== "touch") activate(index, "pointer", false);
      },
      onPointerleave: (event) => {
        if (event.pointerType !== "touch") context.setActive(null, "pointer");
      },
    }),
    registerElement: (index, element) => {
      if (element instanceof Element) elements.set(index, element);
      else elements.delete(index);
    },
    onKeydown,
    activateNearest,
    deactivate: () => context.setActive(null, "pointer"),
  };
}
