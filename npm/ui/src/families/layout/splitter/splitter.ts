/** Accessible, unstyled resizable panel groups following the WAI-ARIA window splitter pattern. */
export { default as SplitterGroup } from "./splitter-group.vue";
export { default as SplitterHandle } from "./splitter-handle.vue";
export { default as SplitterPanel } from "./splitter-panel.vue";
export {
  clampSplitterLayout,
  isValidSplitterLayout,
  resizeSplitterLayout,
  resolveDefaultSplitterLayout,
  splitterLayoutsEqual,
} from "./splitter-layout.ts";
export { parseSplitterLayout, useSplitterPersistence } from "./splitter-persistence.ts";
export type {
  SplitterDirection,
  SplitterGroupExpose,
  SplitterGroupSlotState,
  SplitterGroupState,
  SplitterHandleExpose,
  SplitterHandleSlotState,
  SplitterHandleState,
  SplitterLayout,
  SplitterOrientation,
  SplitterPanelConstraints,
  SplitterPanelExpose,
  SplitterPanelSlotState,
  SplitterPanelState,
  SplitterPersistedLayout,
  SplitterPersistenceOptions,
  SplitterResizeReason,
  SplitterStorage,
} from "./splitter-types.ts";
