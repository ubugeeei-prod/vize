/** Draggable, resizable floating windows with z-order focus, snapping, minimize/maximize, docks, and persistence. */
export { default as FloatingWindow } from "./floating-window.vue";
export { default as WindowDock } from "./window-dock.vue";
export { default as WindowManager } from "./window-manager.vue";
export {
  bringWindowToFront,
  clampWindowRect,
  emptyWindowLayout,
  moveWindowRect,
  parseWindowLayout,
  resizeWindowRect,
  snapWindowRect,
} from "./window-manager-model.ts";
export type {
  WindowBounds,
  WindowConstraints,
  WindowEdge,
  WindowEntry,
  WindowLayout,
  WindowMode,
  WindowRect,
} from "./window-manager-model.ts";
export { useWindowLayoutPersistence } from "./window-manager-persistence.ts";
export type {
  FloatingWindowSlotState,
  WindowDockFilter,
  WindowDockSlotState,
  WindowHandleProps,
  WindowLayoutPersistenceOptions,
  WindowLayoutStorage,
  WindowManagerSlotState,
  WindowPersistedLayout,
  WindowSummary,
} from "./window-manager-types.ts";
