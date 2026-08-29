/** Native textarea line-wrapping policies. */
export type TextareaWrap = "hard" | "off" | "soft";

/** Values accepted by the native `aria-invalid` attribute. */
export type TextareaAriaInvalid = boolean | "grammar" | "spelling";

/** Methods and state exposed by the textarea component instance. */
export interface TextareaExpose {
  /** Whether the native textarea is currently inside IME composition. */
  readonly composing: boolean;

  /** Current controlled or uncontrolled string value. */
  readonly value: string;

  /** Move focus to the native textarea. */
  readonly focus: (options?: FocusOptions) => void;

  /** Select the current native textarea text. */
  readonly select: () => void;

  /** Set the native text selection range. */
  readonly setSelectionRange: (
    selectionStart: number,
    selectionEnd: number,
    direction?: "backward" | "forward" | "none",
  ) => void;

  /** Request a value update and report whether it differs. */
  readonly setValue: (value: string) => boolean;

  /** Restore the current default value and report whether it changed. */
  readonly reset: () => boolean;
}
