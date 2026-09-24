import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";

/** Shared state and actions for PasswordField parts. */
export interface PasswordFieldContextValue {
  readonly inputId: ComputedRef<string>;
  readonly name: ComputedRef<string | undefined>;
  readonly value: ComputedRef<string>;
  readonly visible: ComputedRef<boolean>;
  readonly capsLock: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly readOnly: ComputedRef<boolean>;
  readonly required: ComputedRef<boolean>;
  readonly autocomplete: ComputedRef<string>;
  readonly ariaLabel: ComputedRef<string | undefined>;
  readonly ariaLabelledby: ComputedRef<string | undefined>;
  readonly ariaDescribedby: ComputedRef<string | undefined>;
  readonly ariaErrormessage: ComputedRef<string | undefined>;
  readonly ariaInvalid: ComputedRef<"grammar" | "spelling" | "true" | undefined>;
  readonly setValue: (value: string) => void;
  readonly setVisible: (visible: boolean) => void;
  readonly setCapsLock: (active: boolean) => void;
  readonly registerInput: (element: HTMLInputElement | null) => void;
  readonly focusInput: () => void;
}

/** Typed context shared by PasswordField, its input, and its visibility toggle. */
export const passwordFieldContext = createContext<PasswordFieldContextValue>("PasswordField");
