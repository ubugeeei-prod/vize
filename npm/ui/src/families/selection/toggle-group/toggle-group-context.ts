import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { PrimitiveElement } from "../../foundations/primitive/primitive.ts";
import type {
  ToggleGroupItemState,
  ToggleGroupOrientation,
  ToggleGroupState,
  ToggleGroupType,
  ToggleGroupValue,
} from "./toggle-group-types.ts";

/** Registered item used by the root for DOM-order roving focus. */
export interface ToggleGroupItemRegistration {
  readonly value: () => string;
  readonly disabled: ComputedRef<boolean>;
  readonly element: Readonly<ShallowRef<PrimitiveElement | null>>;
}

/** Keyboard navigation request made by a ToggleGroup item. */
export type ToggleGroupNavigationIntent = "first" | "last" | "next" | "previous";

/** Shared state and actions for ToggleGroup compound items. */
export interface ToggleGroupContextValue {
  readonly activeValue: Readonly<ShallowRef<string | null>>;
  readonly disabled: ComputedRef<boolean>;
  readonly orientation: ComputedRef<ToggleGroupOrientation>;
  readonly pressedValues: ComputedRef<readonly string[]>;
  readonly rovingFocus: ComputedRef<boolean>;
  readonly state: ComputedRef<ToggleGroupState>;
  readonly type: ComputedRef<ToggleGroupType>;
  readonly value: ComputedRef<ToggleGroupValue>;
  readonly getItemState: (value: string, disabled: boolean) => ToggleGroupItemState;
  readonly isPressed: (value: string) => boolean;
  readonly moveFocus: (
    fromValue: string,
    intent: ToggleGroupNavigationIntent,
    options?: FocusOptions,
  ) => boolean;
  readonly registerItem: (item: ToggleGroupItemRegistration) => () => void;
  readonly requestItemToggle: (value: string, nativeEvent: MouseEvent) => boolean;
  readonly setActiveValue: (value: string) => void;
  readonly syncActiveValue: () => void;
}

export const toggleGroupContext = createContext<ToggleGroupContextValue>("ToggleGroup");
