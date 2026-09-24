import type { NumberFieldFormatOptions } from "./number-field-parser.ts";

/** Values accepted by the native `aria-invalid` attribute. */
export type NumberFieldAriaInvalid = boolean | "grammar" | "spelling";

/** Committed NumberField value: a finite number, or `null` while empty. */
export type NumberFieldValue = number | null;

/** State published through the NumberField `data-state` contract. */
export type NumberFieldState =
  | "disabled"
  | "empty"
  | "in-range"
  | "invalid"
  | "max"
  | "min"
  | "readonly";

/** Direction requested by triggers, keyboard, wheel, and the imperative API. */
export type NumberFieldStepDirection = "increment" | "decrement";

/** Source of a committed NumberField change. */
export type NumberFieldChangeSource =
  | "api"
  | "blur"
  | "enter"
  | "keyboard"
  | "reset"
  | "trigger"
  | "wheel";

/** Public props accepted by the NumberField root. */
export interface NumberFieldProps {
  /**
   * Id of the spinbutton input. `null` and `undefined` select a deterministic fallback.
   * Binding a Field's `fieldProps` here wires the label, description, and error.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Native form field name. When set, a hidden input submits the raw number
   * (not the formatted text) with the owning form.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Id of a form owner outside the component tree.
   *
   * @default undefined
   */
  readonly form?: string;

  /**
   * Controlled value. `undefined` selects uncontrolled behavior; `null` is empty.
   *
   * @default undefined
   */
  readonly modelValue?: NumberFieldValue;

  /**
   * Initial uncontrolled value and the value restored by form reset.
   *
   * @default null
   */
  readonly defaultValue?: NumberFieldValue;

  /**
   * Lower bound. Omit for an unbounded field.
   *
   * @default undefined
   */
  readonly min?: number;

  /**
   * Upper bound. Omit for an unbounded field.
   *
   * @default undefined
   */
  readonly max?: number;

  /**
   * Positive step used by arrow keys, triggers, and the wheel.
   *
   * @default 1 (0.01 when `formatOptions.style` is `"percent"`)
   */
  readonly step?: number;

  /**
   * Positive step used by Page Up and Page Down.
   *
   * @default step * 10
   */
  readonly largeStep?: number;

  /**
   * BCP 47 locale for formatting and parsing. Falls back to the nearest
   * `LocaleProvider`, then the document language, then `en-US`.
   *
   * @default undefined
   */
  readonly locale?: string;

  /**
   * `Intl.NumberFormat` options, for example currency, percent, or unit styles.
   *
   * @default undefined
   */
  readonly formatOptions?: NumberFieldFormatOptions;

  /**
   * Clamp typed values into `[min, max]` when they are committed.
   *
   * @default true
   */
  readonly clampOnCommit?: boolean;

  /**
   * Snap typed values to the nearest step when they are committed.
   *
   * @default false
   */
  readonly snapOnCommit?: boolean;

  /**
   * Opt in to stepping with the mouse wheel while the input is focused.
   *
   * @default false
   */
  readonly allowWheel?: boolean;

  /**
   * Delay before press-and-hold on a trigger starts repeating, in milliseconds.
   *
   * @default 400
   */
  readonly holdDelay?: number;

  /**
   * Interval between repeated steps while a trigger is held, in milliseconds.
   *
   * @default 60
   */
  readonly holdInterval?: number;

  /**
   * Disable editing, focus, triggers, and native form submission.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Keep the input focusable while preventing user changes.
   *
   * @default false
   */
  readonly readOnly?: boolean;

  /**
   * Mark the input as required for native constraint validation.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Accessible name when no label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the input.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the input.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;

  /**
   * Id of the validation error message used while invalid.
   *
   * @default undefined
   */
  readonly ariaErrormessage?: string;

  /**
   * Invalid state announced to assistive technology.
   *
   * @default false
   */
  readonly ariaInvalid?: NumberFieldAriaInvalid;
}

/** Emits published by the NumberField root. */
export interface NumberFieldEmits {
  /** Fired when a commit or step requests a new controlled value. */
  "update:modelValue": [value: NumberFieldValue];

  /** Fired after a distinct value is committed, with the previous value and its source. */
  change: [value: NumberFieldValue, previous: NumberFieldValue, source: NumberFieldChangeSource];
}

/** State exposed to NumberField slots and instances. */
export interface NumberFieldSlotState {
  /** Committed numeric value, or `null` while empty. */
  readonly value: NumberFieldValue;

  /** Locale-formatted committed value, or `""` while empty. */
  readonly formattedValue: string;

  /** Text currently shown in the input, including uncommitted typing. */
  readonly inputText: string;

  /** Normalized lower bound (`-Infinity` when unbounded). */
  readonly min: number;

  /** Normalized upper bound (`Infinity` when unbounded). */
  readonly max: number;

  /** Normalized positive step. */
  readonly step: number;

  /** Normalized positive large step. */
  readonly largeStep: number;

  /** Resolved locale used for formatting and parsing. */
  readonly locale: string;

  /** Whether an increment can still change the value. */
  readonly canIncrement: boolean;

  /** Whether a decrement can still change the value. */
  readonly canDecrement: boolean;

  /** Whether editing and triggers are disabled. */
  readonly disabled: boolean;

  /** Whether user edits are locked while focus remains available. */
  readonly readOnly: boolean;

  /** Whether native required validation is requested. */
  readonly required: boolean;

  /** Whether assistive technology announces the field as invalid. */
  readonly invalid: boolean;

  /** Stable state token for styling and tests. */
  readonly state: NumberFieldState;
}

/** Slots exposed by the NumberField root. */
export interface NumberFieldSlots {
  /** Renders the input, triggers, and any adornments with normalized state. */
  default(props: NumberFieldSlotState): unknown;
}

/** Public instance API of the NumberField root. */
export interface NumberFieldExpose extends NumberFieldSlotState {
  /** Rendered root element. */
  readonly root: HTMLDivElement | null;

  /** Registered spinbutton input, once mounted. */
  readonly input: HTMLInputElement | null;

  /** Focus the spinbutton input. */
  readonly focus: (options?: FocusOptions) => void;

  /** Request a value (clamped, and snapped when `snapOnCommit`) and report whether it changed. */
  readonly setValue: (value: NumberFieldValue) => boolean;

  /** Step up by `count` steps (default 1) and report whether the value changed. */
  readonly increment: (count?: number) => boolean;

  /** Step down by `count` steps (default 1) and report whether the value changed. */
  readonly decrement: (count?: number) => boolean;

  /** Parse and commit the current input text; returns whether the value changed. */
  readonly commit: () => boolean;

  /** Restore the default value and report whether it changed. */
  readonly reset: () => boolean;
}

/** Props accepted by NumberFieldInput. */
export interface NumberFieldInputProps {
  /**
   * Native placeholder text shown while empty.
   *
   * @default undefined
   */
  readonly placeholder?: string;

  /**
   * Native autocomplete hint.
   *
   * @default "off"
   */
  readonly autocomplete?: string;
}

/** Emits published by NumberFieldInput. */
export interface NumberFieldInputEmits {
  /** Fired when typed text is rejected because it cannot become a number. */
  reject: [text: string, nativeEvent: Event];
}

/** Slot state for NumberField trigger contents. */
export interface NumberFieldTriggerSlotState {
  /** Direction this trigger steps. */
  readonly direction: NumberFieldStepDirection;

  /** Whether the trigger cannot currently step (bound reached, disabled, or read-only). */
  readonly disabled: boolean;

  /** Whether a press-and-hold repeat is running. */
  readonly holding: boolean;
}

/** Props accepted by NumberFieldIncrement and NumberFieldDecrement. */
export interface NumberFieldTriggerProps {
  /**
   * Accessible name of the trigger.
   *
   * @default "Increase" or "Decrease"
   */
  readonly ariaLabel?: string;
}

/** Slots exposed by NumberFieldIncrement and NumberFieldDecrement. */
export interface NumberFieldTriggerSlots {
  /** Trigger contents, typically an icon. */
  default?(props: NumberFieldTriggerSlotState): unknown;
}
