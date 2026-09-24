import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { DashboardItem } from "./dashboard-grid-layout.ts";

/** Shared state between `DashboardGrid` and its widgets. */
export interface DashboardGridContextValue {
  readonly itemOf: (id: string, fallback: DashboardItem) => DashboardItem;
  readonly disabled: ComputedRef<boolean>;
  readonly cellSize: () => { readonly width: number; readonly height: number };
  readonly move: (item: DashboardItem, x: number, y: number) => void;
  readonly resize: (item: DashboardItem, w: number, h: number) => void;
}

export const dashboardGridContext = createContext<DashboardGridContextValue>("DashboardGrid");
