import "./progress-bar.css";

export {
  PROGRESS_BAR_DEFAULT_MAX,
  PROGRESS_BAR_DEFAULT_MIN,
  getProgressBarState,
  getProgressBarStyle,
} from "./progress-bar-state.ts";

/** Accessible progressbar primitive for determinate and indeterminate work. */
export { default as ProgressBar } from "./progress-bar.vue";

export type {
  ProgressBarDirection,
  ProgressBarExpose,
  ProgressBarProps,
  ProgressBarSlots,
  ProgressBarSlotState,
  ProgressBarState,
  ProgressBarStyle,
} from "./progress-bar-types.ts";
