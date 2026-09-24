import type { MenuRootExpose, MenuSlotState } from "../menu/menu-types.ts";

/** Viewport point a context menu is anchored to. */
export interface ContextMenuPoint {
  /** Horizontal viewport coordinate in CSS pixels. */
  readonly x: number;

  /** Vertical viewport coordinate in CSS pixels. */
  readonly y: number;
}

/** State exposed to ContextMenuTrigger slots. */
export interface ContextMenuTriggerSlotState extends MenuSlotState {
  /** Whether the trigger ignores context-menu requests (the native menu shows instead). */
  readonly disabled: boolean;
}

/** Imperative surface exposed by ContextMenuRoot. */
export interface ContextMenuRootExpose extends MenuRootExpose {
  /** Open the menu anchored at a viewport point, as a context-menu request would. */
  readonly openAt: (point: ContextMenuPoint, event?: Event | null) => boolean;
}

/** Imperative surface exposed by ContextMenuTrigger. */
export interface ContextMenuTriggerExpose {
  /** Rendered trigger region. */
  readonly element: HTMLSpanElement | null;
}
