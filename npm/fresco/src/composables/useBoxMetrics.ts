/**
 * useBoxMetrics - best-effort layout metrics for a rendered Fresco node.
 */

import { onMounted, onUnmounted, reactive, type Ref } from "vue";
import { getLastRenderLayout, updateLastRenderLayouts, type DOMElement } from "../layoutMetrics.js";

/** Last measured layout rectangle in terminal cells. */
export interface BoxMetrics {
  width: number;
  height: number;
  left: number;
  top: number;
}

/** Measured rectangle, measurement-state flag, and an explicit refresh request. */
export interface UseBoxMetricsResult extends BoxMetrics {
  hasMeasured: boolean;
  /** Refresh from the latest completed native layout snapshot. */
  refresh(): void;
}

/** Host node or component element accepted by terminal layout measurement. */
export type MetricTarget = DOMElement;

async function loadNative() {
  return import("@vizejs/fresco-native");
}

function targetId(target: MetricTarget | null | undefined): number | undefined {
  return target?.id ?? target?.$el?.id;
}

/** Read completed layout measurements, falling back to native measurement; poll after mount and stop on unmount. */
export function useBoxMetrics(ref: Ref<MetricTarget | null>): UseBoxMetricsResult {
  const metrics = reactive({
    width: 0,
    height: 0,
    left: 0,
    top: 0,
    hasMeasured: false,
  }) as UseBoxMetricsResult;

  let timer: ReturnType<typeof setInterval> | null = null;

  const applyLayout = (layout: { width: number; height: number; x: number; y: number }) => {
    metrics.width = layout.width;
    metrics.height = layout.height;
    metrics.left = layout.x;
    metrics.top = layout.y;
    metrics.hasMeasured = true;
  };

  const refresh = () => {
    const id = targetId(ref.value);
    if (id === undefined) {
      metrics.width = 0;
      metrics.height = 0;
      metrics.left = 0;
      metrics.top = 0;
      metrics.hasMeasured = false;
      return;
    }

    const cachedLayout = getLastRenderLayout(ref.value);
    if (cachedLayout) {
      applyLayout(cachedLayout);
      return;
    }

    void loadNative()
      .then((native) => {
        const layouts = "getLastRenderLayouts" in native ? native.getLastRenderLayouts() : [];
        updateLastRenderLayouts(layouts);
        const layout = layouts.find((item: { id: number }) => item.id === id);
        if (!layout) return;

        applyLayout(layout);
      })
      .catch(() => {});
  };

  metrics.refresh = refresh;

  onMounted(() => {
    refresh();
    timer = setInterval(refresh, 100);
  });

  onUnmounted(() => {
    if (timer) clearInterval(timer);
  });

  return metrics;
}
