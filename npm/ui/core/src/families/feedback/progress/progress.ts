export { PROGRESS_DEFAULT_MAX, getProgressState } from "./progress-state.ts";

/** Accessible, unstyled native progressbar for determinate and indeterminate work. */
export { default as Progress } from "./progress.vue";

export type {
  ProgressExpose,
  ProgressProps,
  ProgressSlots,
  ProgressSlotState,
  ProgressState,
} from "./progress-types.ts";
