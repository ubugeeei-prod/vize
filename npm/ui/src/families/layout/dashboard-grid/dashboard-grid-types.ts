import type { DashboardItem } from "./dashboard-grid-layout.ts";

/** Props to spread on a widget's drag handle. */
export interface DashboardHandleProps {
  readonly "data-part": "drag-handle";
  readonly tabindex: 0;
  readonly "aria-keyshortcuts": string;
  readonly onPointerdown: (event: PointerEvent) => void;
  readonly onKeydown: (event: KeyboardEvent) => void;
}

/** Props to spread on a widget's resize handle. */
export interface DashboardResizeHandleProps {
  readonly "data-part": "resize-handle";
  readonly "aria-hidden": "true";
  readonly onPointerdown: (event: PointerEvent) => void;
}

/** Slot props of `DashboardGridItem`. */
export interface DashboardGridItemSlotState {
  /** Current placement (default placement until the layout contains the widget). */
  readonly item: DashboardItem;
  /** Whether the widget is being dragged or resized. */
  readonly dragging: boolean;
  /** Spread on the drag handle: drag or arrows to move, Shift+arrows to resize. */
  readonly handleProps: DashboardHandleProps;
  /** Spread on a resize grip (bottom-right corner). */
  readonly resizeHandleProps: DashboardResizeHandleProps;
}

/** Slot props of `DashboardGrid`. */
export interface DashboardGridSlotState {
  /** Rows currently occupied. */
  readonly rows: number;
}
