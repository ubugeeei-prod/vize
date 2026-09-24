import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { EditableActivationMode, EditableState } from "./editable-types.ts";

/** Shared state and actions for Editable parts. */
export interface EditableContextValue {
  readonly inputId: ComputedRef<string>;
  readonly value: ComputedRef<string>;
  readonly draft: ComputedRef<string>;
  readonly editing: ComputedRef<boolean>;
  readonly interactive: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly readOnly: ComputedRef<boolean>;
  readonly state: ComputedRef<EditableState>;
  readonly placeholder: ComputedRef<string | undefined>;
  readonly maxLength: ComputedRef<number | undefined>;
  readonly activationMode: ComputedRef<EditableActivationMode>;
  readonly submitOnBlur: ComputedRef<boolean>;
  readonly submitOnEnter: ComputedRef<boolean>;
  readonly ariaLabel: ComputedRef<string | undefined>;
  readonly ariaLabelledby: ComputedRef<string | undefined>;
  readonly ariaDescribedby: ComputedRef<string | undefined>;
  readonly ariaErrormessage: ComputedRef<string | undefined>;
  readonly ariaInvalid: ComputedRef<"grammar" | "spelling" | "true" | undefined>;
  readonly setDraft: (value: string) => void;
  readonly edit: () => boolean;
  readonly submit: () => boolean;
  readonly cancel: () => void;
  readonly registerInput: (element: HTMLInputElement | null) => void;
  readonly registerPreview: (element: HTMLElement | null) => void;
  readonly consumeFocusReturn: () => boolean;
}

/** Typed context shared by Editable, its preview, input, and triggers. */
export const editableContext = createContext<EditableContextValue>("Editable");
