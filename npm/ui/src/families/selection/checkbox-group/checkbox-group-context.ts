import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { CheckboxGroupItemState } from "./checkbox-group-types.ts";

/**
 * Type-erased group state shared with items. Items resolve their `value` to an
 * option index by identity, so the root never needs to accept `unknown` values.
 */
export interface CheckboxGroupContextValue {
  readonly name: ComputedRef<string | undefined>;
  readonly form: ComputedRef<string | undefined>;
  readonly required: ComputedRef<boolean>;
  readonly groupDisabled: ComputedRef<boolean>;
  readonly hasSelection: ComputedRef<boolean>;
  readonly selectAllState: ComputedRef<CheckboxGroupItemState>;
  readonly selectAllDisabled: ComputedRef<boolean>;
  readonly indexOf: (candidate: unknown) => number;
  readonly isIndexSelected: (index: number) => boolean;
  readonly isIndexDisabled: (index: number) => boolean;
  readonly formValueOf: (index: number) => string;
  readonly toggleIndex: (index: number, selected: boolean) => boolean;
  readonly toggleAll: (selected: boolean) => boolean;
  readonly ariaDescribedby: ComputedRef<string | undefined>;
  readonly ariaInvalid: ComputedRef<"grammar" | "spelling" | "true" | undefined>;
  readonly ariaErrormessage: ComputedRef<string | undefined>;
}

/** Typed context shared by CheckboxGroup, its items, and the select-all parent. */
export const checkboxGroupContext = createContext<CheckboxGroupContextValue>("CheckboxGroup");
