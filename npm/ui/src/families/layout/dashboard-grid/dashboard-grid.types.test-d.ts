/** Compile-only assertions for the dashboard grid. */

import {
  compactDashboardLayout,
  dashboardItemsCollide,
  moveDashboardItem,
  resizeDashboardItem,
} from "./dashboard-grid.ts";
import type {
  DashboardCompaction,
  DashboardGridItemSlotState,
  DashboardItem,
  DashboardLayout,
} from "./dashboard-grid.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

type _CompactionIsClosed = Expect<Equal<DashboardCompaction, "vertical" | "none">>;
const layout: DashboardLayout = [{ id: "a", x: 0, y: 0, w: 1, h: 1, static: true }];
type _MoveReturnsALayout = Expect<Equal<ReturnType<typeof moveDashboardItem>, DashboardLayout>>;
resizeDashboardItem(layout, "a", 2, 2, 12, "none") satisfies DashboardLayout;
compactDashboardLayout(layout) satisfies DashboardItem[];
dashboardItemsCollide(layout[0] as DashboardItem, layout[0] as DashboardItem) satisfies boolean;
// @ts-expect-error compaction strategies are closed.
compactDashboardLayout(layout, "horizontal");
// @ts-expect-error items need cell geometry.
const _missing: DashboardItem = { id: "b", x: 0, y: 0 };
// @ts-expect-error layouts are readonly.
layout.push({ id: "c", x: 0, y: 0, w: 1, h: 1 });

declare const slot: DashboardGridItemSlotState;
type _SlotItem = Expect<Equal<typeof slot.item, DashboardItem>>;
slot.handleProps.onKeydown satisfies (event: KeyboardEvent) => void;
type _ResizeHandleIsHidden = Expect<Equal<(typeof slot.resizeHandleProps)["aria-hidden"], "true">>;
