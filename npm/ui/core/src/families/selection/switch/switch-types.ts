/** Values accepted by the native `aria-invalid` attribute. */
export type SwitchAriaInvalid = boolean | "grammar" | "spelling";

/** State exposed through the switch Native CSS data contract. */
export type SwitchState = "checked" | "disabled" | "readonly" | "unchecked";

/** State exposed to the default slot. */
export interface SwitchSlotState {
  /** Whether the switch is currently on. */
  readonly checked: boolean;

  /** Whether native activation and form submission are disabled. */
  readonly disabled: boolean;

  /** Whether the switch is focusable but user activation is locked. */
  readonly readOnly: boolean;

  /** Whether assistive technology should announce the switch as required. */
  readonly required: boolean;

  /** Whether the switch is currently marked invalid. */
  readonly invalid: boolean;
}

/** Methods and state exposed by the switch component instance. */
export interface SwitchExpose {
  /** Current controlled or uncontrolled checked state. */
  readonly checked: boolean;

  /** Move focus to the native switch button. */
  readonly focus: (options?: FocusOptions) => void;

  /** Request the opposite checked state and report whether it differs. */
  readonly toggle: () => boolean;

  /** Request a checked-state update and report whether it differs. */
  readonly setChecked: (value: boolean) => boolean;

  /** Restore the current default checked state and report whether it changed. */
  readonly reset: () => boolean;
}
