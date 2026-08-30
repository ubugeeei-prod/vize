export {
  PROGRESS_DEFAULT_MAX,
  getProgressState,
} from "./families/feedback/progress/progress-state.ts";

/** Backward-compatible native Progress primitive. */
export { default as Progress } from "./families/feedback/progress/progress.vue";

export type {
  ProgressExpose,
  ProgressProps,
  ProgressSlots,
  ProgressSlotState,
  ProgressState,
} from "./families/feedback/progress/progress-types.ts";
