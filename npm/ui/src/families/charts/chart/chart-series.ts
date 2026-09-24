import { computed, watch } from "vue";
import type { ComputedRef } from "vue";

import type { ChartContextValue } from "./chart-context.ts";
import type { ChartSeriesKind } from "./chart-types.ts";

/**
 * Register a named series with its chart for legends and data tables, and
 * report whether the legend currently hides it. Unnamed series stay anonymous
 * and always render.
 */
export function useChartSeries(
  context: ChartContextValue,
  name: () => string | undefined,
  kind: ChartSeriesKind,
): ComputedRef<boolean> {
  watch(
    name,
    (next, _previous, onCleanup) => {
      if (next !== undefined) onCleanup(context.registerSeries(next, kind));
    },
    { flush: "sync", immediate: true },
  );
  return computed(() => {
    const current = name();
    return current !== undefined && context.isHidden(current);
  });
}
