import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { NumberFieldParser } from "./number-field-parser.ts";
import type { NumberFieldBounds } from "./number-field-state.ts";
import type {
  NumberFieldAriaInvalid,
  NumberFieldChangeSource,
  NumberFieldState,
  NumberFieldStepDirection,
  NumberFieldValue,
} from "./number-field-types.ts";

/** Shared state and actions for NumberField parts. */
export interface NumberFieldContextValue {
  readonly inputId: ComputedRef<string>;
  readonly form: ComputedRef<string | undefined>;
  readonly value: ComputedRef<NumberFieldValue>;
  readonly inputText: ComputedRef<string>;
  readonly setDraft: (text: string) => void;
  readonly bounds: ComputedRef<NumberFieldBounds>;
  readonly parser: ComputedRef<NumberFieldParser>;
  readonly state: ComputedRef<NumberFieldState>;
  readonly disabled: ComputedRef<boolean>;
  readonly readOnly: ComputedRef<boolean>;
  readonly required: ComputedRef<boolean>;
  readonly invalid: ComputedRef<boolean>;
  readonly allowWheel: ComputedRef<boolean>;
  readonly holdDelay: ComputedRef<number>;
  readonly holdInterval: ComputedRef<number>;
  readonly ariaInvalid: ComputedRef<Exclude<NumberFieldAriaInvalid, boolean> | "true" | undefined>;
  readonly ariaLabel: ComputedRef<string | undefined>;
  readonly ariaLabelledby: ComputedRef<string | undefined>;
  readonly ariaDescribedby: ComputedRef<string | undefined>;
  readonly ariaErrormessage: ComputedRef<string | undefined>;
  readonly inputElement: ShallowRef<HTMLInputElement | null>;
  readonly canStep: (direction: NumberFieldStepDirection) => boolean;
  readonly step: (
    direction: NumberFieldStepDirection,
    size: "step" | "largeStep",
    source: NumberFieldChangeSource,
  ) => boolean;
  readonly stepToBound: (bound: "min" | "max", source: NumberFieldChangeSource) => boolean;
  readonly commit: (source: NumberFieldChangeSource) => boolean;
}

/** Typed context shared by NumberField, NumberFieldInput, and the triggers. */
export const numberFieldContext = createContext<NumberFieldContextValue>("NumberField");
