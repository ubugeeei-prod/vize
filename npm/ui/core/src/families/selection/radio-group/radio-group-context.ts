import type { ComputedRef } from "vue";

import { createContext } from "../../../context.ts";
import type {
  RadioGroupOrientation,
  RadioGroupState,
  RadioGroupValue,
} from "./radio-group-types.ts";

/** Shared state and native form attributes for RadioGroup compound items. */
export interface RadioGroupContextValue {
  readonly id: ComputedRef<string>;
  readonly name: ComputedRef<string | undefined>;
  readonly value: ComputedRef<RadioGroupValue>;
  readonly disabled: ComputedRef<boolean>;
  readonly required: ComputedRef<boolean>;
  readonly invalid: ComputedRef<boolean>;
  readonly orientation: ComputedRef<RadioGroupOrientation>;
  readonly state: ComputedRef<RadioGroupState>;
  readonly selectValue: (value: string, nativeEvent: Event) => boolean;
}

export const radioGroupContext = createContext<RadioGroupContextValue>("RadioGroup");
