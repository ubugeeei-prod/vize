/** Text-like native input types handled by the Input primitive. */
export type InputType = "email" | "password" | "search" | "tel" | "text" | "url";

/** Native virtual-keyboard input modes accepted by the Input primitive. */
export type InputInputMode =
  | "decimal"
  | "email"
  | "none"
  | "numeric"
  | "search"
  | "tel"
  | "text"
  | "url";

/** Native virtual-keyboard enter-key labels accepted by the Input primitive. */
export type InputEnterKeyHint = "done" | "enter" | "go" | "next" | "previous" | "search" | "send";

/** Values accepted by the native `aria-invalid` attribute. */
export type InputAriaInvalid = boolean | "grammar" | "spelling";

/** Methods and state exposed by the input component instance. */
export interface InputExpose {
  /** Whether the native input is currently inside IME composition. */
  readonly composing: boolean;

  /** Current controlled or uncontrolled string value. */
  readonly value: string;

  /** Move focus to the native input. */
  readonly focus: (options?: FocusOptions) => void;

  /** Select the current native input text. */
  readonly select: () => void;

  /** Request a value update and report whether it differs. */
  readonly setValue: (value: string) => boolean;

  /** Restore the current default value and report whether it changed. */
  readonly reset: () => boolean;
}
