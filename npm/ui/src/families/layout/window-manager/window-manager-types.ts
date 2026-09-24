import type { ShallowRef } from "vue";

import type { WindowEntry, WindowLayout, WindowMode } from "./window-manager-model.ts";

/** Synchronous storage used by {@link useWindowLayoutPersistence}. */
export interface WindowLayoutStorage {
  /** Read a stored value. */
  getItem(key: string): string | null;
  /** Persist a value. */
  setItem(key: string, value: string): void;
}

/** Options for {@link useWindowLayoutPersistence}. */
export interface WindowLayoutPersistenceOptions {
  /** Storage key. */
  readonly key: string;
  /**
   * Storage adapter (or a getter resolved after mount).
   *
   * @default globalThis.localStorage when available
   */
  readonly storage?: WindowLayoutStorage | (() => WindowLayoutStorage | null | undefined);
  /**
   * Read during setup instead of after mount (only hydration-safe for storage
   * the server can read, such as cookies).
   *
   * @default false
   */
  readonly immediate?: boolean;
}

/** Ref bound with `v-model:layout` on `WindowManager`. */
export type WindowPersistedLayout = ShallowRef<WindowLayout | undefined>;

/** Summary of one window for docks and taskbars. */
export interface WindowSummary {
  /** Window id. */
  readonly id: string;
  /** Accessible title. */
  readonly title: string;
  /** Display mode. */
  readonly mode: WindowMode;
  /** Whether the window is frontmost. */
  readonly active: boolean;
  /** Restore (if minimized) and bring to front. */
  readonly activate: () => void;
  /** Minimize the window. */
  readonly minimize: () => void;
}

/** Props to spread on a window's drag handle (title bar). */
export interface WindowHandleProps {
  readonly "data-part": "drag-handle";
  readonly tabindex: 0;
  readonly "aria-keyshortcuts": string;
  readonly onPointerdown: (event: PointerEvent) => void;
  readonly onKeydown: (event: KeyboardEvent) => void;
}

/** Slot props of `FloatingWindow`. */
export interface FloatingWindowSlotState {
  /** Current geometry and mode. */
  readonly state: WindowEntry;
  /** Whether the window is frontmost. */
  readonly active: boolean;
  /** Spread on the title bar: drag to move, arrows to move, Shift+arrows to resize. */
  readonly handleProps: WindowHandleProps;
  /** Minimize the window. */
  readonly minimize: () => void;
  /** Maximize the window. */
  readonly maximize: () => void;
  /** Restore a minimized or maximized window. */
  readonly restore: () => void;
  /** Toggle between maximized and normal. */
  readonly toggleMaximize: () => void;
  /** Request closing (emits `close`). */
  readonly close: () => void;
}

/** Slot props of `WindowManager`. */
export interface WindowManagerSlotState {
  /** Frontmost window id, or `null`. */
  readonly activeId: string | null;
  /** Summaries of every registered window. */
  readonly windows: readonly WindowSummary[];
}

/** Slot props of `WindowDock`. */
export interface WindowDockSlotState {
  /** Summaries of the windows shown in the dock. */
  readonly windows: readonly WindowSummary[];
}

/** Which windows `WindowDock` lists. */
export type WindowDockFilter = "all" | "minimized";
