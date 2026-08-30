import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../../context.ts";
import type { PrimitiveElement } from "../../../primitive.ts";
import type {
  ButtonGroupItemState,
  ButtonGroupOrientation,
  ButtonGroupRole,
  ButtonGroupState,
} from "./button-group-types.ts";

/** Registered item used by the root for DOM-order roving focus. */
export interface ButtonGroupItemRegistration {
  readonly value: () => string;
  readonly disabled: ComputedRef<boolean>;
  readonly element: Readonly<ShallowRef<PrimitiveElement | null>>;
}

/** Keyboard navigation request made by a ButtonGroup item. */
export type ButtonGroupNavigationIntent = "first" | "last" | "next" | "previous";

/** Shared state and actions for ButtonGroup compound items. */
export interface ButtonGroupContextValue {
  readonly activeValue: Readonly<ShallowRef<string | null>>;
  readonly disabled: ComputedRef<boolean>;
  readonly orientation: ComputedRef<ButtonGroupOrientation>;
  readonly role: ComputedRef<ButtonGroupRole>;
  readonly rovingFocus: ComputedRef<boolean>;
  readonly state: ComputedRef<ButtonGroupState>;
  readonly getItemState: (disabled: boolean) => ButtonGroupItemState;
  readonly moveFocus: (
    fromValue: string,
    intent: ButtonGroupNavigationIntent,
    options?: FocusOptions,
  ) => boolean;
  readonly registerItem: (item: ButtonGroupItemRegistration) => () => void;
  readonly requestItemPress: (value: string, nativeEvent: MouseEvent) => void;
  readonly setActiveValue: (value: string) => void;
  readonly syncActiveValue: (item?: ButtonGroupItemRegistration) => void;
}

export const buttonGroupContext = createContext<ButtonGroupContextValue>("ButtonGroup");
