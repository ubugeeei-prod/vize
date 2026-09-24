import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  ChartActivePoint,
  ChartActiveReason,
  ChartMargin,
  ChartSeriesInfo,
  ChartSeriesKind,
} from "./chart-types.ts";

/** Shared geometry, series registry, and focus state for chart parts. */
export interface ChartContextValue {
  readonly id: ComputedRef<string>;
  readonly width: ComputedRef<number>;
  readonly height: ComputedRef<number>;
  readonly innerWidth: ComputedRef<number>;
  readonly innerHeight: ComputedRef<number>;
  readonly margin: ComputedRef<ChartMargin>;
  readonly series: ComputedRef<readonly ChartSeriesInfo[]>;
  readonly active: ComputedRef<ChartActivePoint | null>;
  readonly registerSeries: (name: string, kind: ChartSeriesKind) => () => void;
  readonly isHidden: (name: string) => boolean;
  readonly toggleSeries: (name: string, hidden?: boolean) => boolean;
  readonly setActive: (point: ChartActivePoint | null, reason: ChartActiveReason) => void;
  readonly announce: (text: string) => void;
  readonly plotPoint: (event: MouseEvent) => { readonly x: number; readonly y: number } | null;
}

export const chartContext = createContext<ChartContextValue>("Chart");
