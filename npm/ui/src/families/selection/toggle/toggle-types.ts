import type { ComputedRef } from "vue";

/** State exposed to the default slot. */
export interface ToggleSlotState {
  /** Whether the toggle is unavailable for user activation. */
  readonly disabled: boolean;

  /** Whether the toggle is currently pressed. */
  readonly pressed: boolean;
}

/** Methods and state exposed by the toggle component instance. */
export interface ToggleExpose {
  /** Current controlled or uncontrolled pressed state. */
  readonly pressed: ComputedRef<boolean>;

  /** Move focus to the rendered control when supported. */
  readonly focus: (options?: FocusOptions) => void;

  /** Request a pressed-state update and report whether it differs. */
  readonly setPressed: (value: boolean) => boolean;

  /** Restore the current default pressed state and report whether it changed. */
  readonly reset: () => boolean;
}
