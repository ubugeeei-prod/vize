/** Values accepted by the native `aria-invalid` attribute. */
export type SearchFieldAriaInvalid = boolean | "grammar" | "spelling";

/** Clear button visibility policy for the search field. */
export type SearchFieldClearVisibility = "always" | "auto" | "never";

/** Native virtual-keyboard enter-key labels accepted by the SearchField primitive. */
export type SearchFieldEnterKeyHint =
  | "done"
  | "enter"
  | "go"
  | "next"
  | "previous"
  | "search"
  | "send";

/** Native virtual-keyboard input modes accepted by the SearchField primitive. */
export type SearchFieldInputMode =
  | "decimal"
  | "email"
  | "none"
  | "numeric"
  | "search"
  | "tel"
  | "text"
  | "url";

/** Props accepted by the SearchField component. */
export interface SearchFieldProps {
  /** Consumer-owned input id. `null` and `undefined` select a deterministic fallback. */
  readonly id?: string | null;

  /** Native form field name. */
  readonly name?: string;

  /** Controlled string value. `undefined` selects uncontrolled behavior. */
  readonly modelValue?: string;

  /** Initial value for uncontrolled use and the value restored by form reset. */
  readonly defaultValue?: string;

  /** Disable editing, clearing, focus, and native form submission. */
  readonly disabled?: boolean;

  /** Keep the search input focusable while preventing user editing and clearing. */
  readonly readOnly?: boolean;

  /** Mark the search input as required for native constraint validation. */
  readonly required?: boolean;

  /** Native placeholder text. */
  readonly placeholder?: string;

  /** Native autocomplete hint. */
  readonly autocomplete?: string;

  /** Native virtual-keyboard input mode. */
  readonly inputMode?: SearchFieldInputMode;

  /** Native virtual-keyboard enter key hint. */
  readonly enterKeyHint?: SearchFieldEnterKeyHint;

  /** Clear button visibility policy. */
  readonly showClear?: SearchFieldClearVisibility;

  /** Accessible name for the default clear button. */
  readonly clearLabel?: string;

  /** Accessible name when no label or `aria-labelledby` supplies one. */
  readonly ariaLabel?: string;

  /** Space-separated ids that label the search input. */
  readonly ariaLabelledby?: string;

  /** Space-separated ids that describe the search input. */
  readonly ariaDescribedby?: string;

  /** Id of the validation error message used while invalid. */
  readonly ariaErrormessage?: string;

  /** Invalid state announced to assistive technology. */
  readonly ariaInvalid?: SearchFieldAriaInvalid;
}

/** State exposed to the clear slot. */
export interface SearchFieldClearSlotState {
  /** Whether clear activation is currently unavailable. */
  readonly disabled: boolean;

  /** Whether the current value is empty. */
  readonly empty: boolean;
}

/** Slots accepted by the SearchField component. */
export interface SearchFieldSlots {
  /** Renders the default clear button contents with current availability state. */
  clear(props: SearchFieldClearSlotState): unknown;
}

/** Emits declared by the SearchField component. */
export interface SearchFieldEmits {
  /** Fired when the value requests a new controlled string. */
  "update:modelValue": [value: string];

  /** Fired after the default clear button clears the field. */
  clear: [value: "", nativeEvent: MouseEvent];

  /** Fired after a native change/commit with the current string and native `Event`. */
  change: [value: string, nativeEvent: Event];

  /** Fired when IME composition ends. */
  compositionEnd: [value: string, nativeEvent: CompositionEvent];

  /** Fired when IME composition starts. */
  compositionStart: [value: string, nativeEvent: CompositionEvent];

  /** Fired after a native input event with the next string and native `Event`. */
  input: [value: string, nativeEvent: Event];

  /** Fired after a native search event with the committed search string. */
  search: [value: string, nativeEvent: Event];
}

/** Methods and state exposed by the search field component instance. */
export interface SearchFieldExpose {
  /** Whether the native input is currently inside IME composition. */
  readonly composing: boolean;

  /** Current controlled or uncontrolled string value. */
  readonly value: string;

  /** Clear the current value and report whether it changed. */
  readonly clear: () => boolean;

  /** Move focus to the native search input. */
  readonly focus: (options?: FocusOptions) => void;

  /** Restore the current default value and report whether it changed. */
  readonly reset: () => boolean;

  /** Select the current native search text. */
  readonly select: () => void;

  /** Request a value update and report whether it differs. */
  readonly setValue: (value: string) => boolean;
}
