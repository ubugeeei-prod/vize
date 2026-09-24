import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { PinInputType } from "./pin-input-types.ts";

/** Shared state and actions for PinInput fields. */
export interface PinInputContextValue {
  readonly baseId: ComputedRef<string>;
  readonly characters: ComputedRef<readonly string[]>;
  readonly length: ComputedRef<number>;
  readonly type: ComputedRef<PinInputType>;
  readonly mask: ComputedRef<boolean>;
  readonly otp: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly required: ComputedRef<boolean>;
  readonly complete: ComputedRef<boolean>;
  readonly placeholder: ComputedRef<string | undefined>;
  readonly ariaDescribedby: ComputedRef<string | undefined>;
  readonly ariaInvalid: ComputedRef<"grammar" | "spelling" | "true" | undefined>;
  readonly ariaErrormessage: ComputedRef<string | undefined>;
  readonly fieldLabel: (index: number) => string;
  readonly registerField: (index: number, element: HTMLInputElement | null) => void;
  readonly focusField: (index: number) => void;
  readonly write: (index: number, text: string) => number;
  readonly remove: (index: number, direction: "backward" | "forward") => number;
}

/** Typed context shared by PinInput and its fields. */
export const pinInputContext = createContext<PinInputContextValue>("PinInput");
