/** Drag-and-resize dashboard widgets on a CSS grid with pure, collision-resolving layout functions. */
export { default as DashboardGrid } from "./dashboard-grid.vue";
export { default as DashboardGridItem } from "./dashboard-grid-item.vue";
export {
  compactDashboardLayout,
  createDashboardItem,
  dashboardItemsCollide,
  dashboardLayoutRows,
  moveDashboardItem,
  normalizeDashboardLayout,
  resizeDashboardItem,
} from "./dashboard-grid-layout.ts";
export type {
  DashboardCompaction,
  DashboardItem,
  DashboardLayout,
} from "./dashboard-grid-layout.ts";
export type {
  DashboardGridItemSlotState,
  DashboardGridSlotState,
  DashboardHandleProps,
  DashboardResizeHandleProps,
} from "./dashboard-grid-types.ts";
