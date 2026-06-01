/**
 * useBoxMetrics - best-effort layout metrics for a rendered Fresco node.
 */

import { onMounted, onUnmounted, reactive, type Ref } from "@vue/runtime-core";
import { getLastRenderLayout, updateLastRenderLayouts, type DOMElement } from "../layoutMetrics.js";

export interface BoxMetrics {
  width: number;
  height: number;
  left: number;
  top: number;
}

export interface UseBoxMetricsResult extends BoxMetrics {
  hasMeasured: boolean;
}

export type MetricTarget = DOMElement;

async function loadNative() {
  return import("@vizejs/fresco-native");
}

function targetId(target: MetricTarget | null | undefined): number | undefined {
  return target?.id ?? target?.$el?.id;
}

export function useBoxMetrics(ref: Ref<MetricTarget | null>): UseBoxMetricsResult {
  const metrics = reactive<UseBoxMetricsResult>({
    width: 0,
    height: 0,
    left: 0,
    top: 0,
    hasMeasured: false,
  });

  let timer: ReturnType<typeof setInterval> | null = null;

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

    void loadNative()
      .then((native) => {
        const layouts = "getLastRenderLayouts" in native ? native.getLastRenderLayouts() : [];
        updateLastRenderLayouts(layouts);
        const layout = layouts.find((item: { id: number }) => item.id === id);
        if (!layout) return;

        metrics.width = layout.width;
        metrics.height = layout.height;
        metrics.left = layout.x;
        metrics.top = layout.y;
        metrics.hasMeasured = true;
      })
      .catch(() => {});
  };

  onMounted(() => {
    const layout = getLastRenderLayout(ref.value);
    if (layout) {
      metrics.width = layout.width;
      metrics.height = layout.height;
      metrics.left = layout.x;
      metrics.top = layout.y;
      metrics.hasMeasured = true;
    }

    refresh();
    timer = setInterval(refresh, 100);
  });

  onUnmounted(() => {
    if (timer) clearInterval(timer);
  });

  return metrics;
}
