import type { InputMaskResult, InputMaskTokens } from "./input-mask-engine.ts";
import type { InputMaskValueFormat } from "./input-mask-runtime.ts";

/** Values accepted by the native `aria-invalid` attribute. */
export type MaskedInputAriaInvalid = boolean | "grammar" | "spelling";

/** State published through the MaskedInput `data-state` contract. */
export type MaskedInputState = "complete" | "disabled" | "empty" | "incomplete" | "readonly";

/** Public props accepted by MaskedInput. */
export interface MaskedInputProps {
  /**
   * Mask pattern. `9` digit, `a` letter, `*` letter or digit, `\` escapes a literal.
   *
   * @default required
   */
  readonly mask: string;

  /**
   * Extra or overriding tokens (see `defineInputMaskTokens`).
   *
   * @default undefined
   */
  readonly tokens?: InputMaskTokens;

  /**
   * Placeholder character for unfilled slots when `lazy` is `false`.
   *
   * @default "_"
   */
  readonly placeholderChar?: string;

  /**
   * Hide unfilled slots; `false` shows the whole mask with placeholders.
   *
   * @default true
   */
  readonly lazy?: boolean;

  /**
   * Append literals right after the last filled slot while typing.
   *
   * @default false
   */
  readonly eager?: boolean;

  /**
   * Representation used by `modelValue`, `defaultValue`, and emits.
   *
   * @default "raw"
   */
  readonly valueFormat?: InputMaskValueFormat;

  /**
   * Controlled model value. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: string;

  /**
   * Initial uncontrolled value, also restored by form reset.
   *
   * @default ""
   */
  readonly defaultValue?: string;

  /**
   * Consumer-owned input id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Native form field name; the displayed (masked) text is submitted.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Disable editing, focus, and native form submission.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Keep the input focusable while preventing edits.
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
   * Native placeholder shown while empty.
   *
   * @default undefined
   */
  readonly placeholder?: string;

  /**
   * Native autocomplete hint.
   *
   * @default undefined
   */
  readonly autocomplete?: string;

  /**
   * Virtual keyboard hint. Defaults to `"numeric"` for digit-only masks.
   *
   * @default undefined
   */
  readonly inputMode?: "decimal" | "email" | "numeric" | "search" | "tel" | "text" | "url";

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
  readonly ariaInvalid?: MaskedInputAriaInvalid;
}

/** Emits published by MaskedInput. */
export interface MaskedInputEmits {
  /** Fired when the model value (in `valueFormat`) changes. */
  "update:modelValue": [value: string];

  /** Fired when the last slot is filled, with the conform result. */
  complete: [result: InputMaskResult];
}

/** Public instance API of MaskedInput. */
export interface MaskedInputExpose {
  /** Rendered native input. */
  readonly element: HTMLInputElement | null;

  /** Displayed text. */
  readonly masked: string;

  /** Characters accepted into slots. */
  readonly raw: string;

  /** Whether every slot is filled. */
  readonly complete: boolean;

  /** Move focus to the input. */
  readonly focus: (options?: FocusOptions) => void;

  /** Conform and store text; returns whether the model changed. */
  readonly setValue: (text: string) => boolean;

  /** Restore the default value; returns whether the model changed. */
  readonly reset: () => boolean;
}
