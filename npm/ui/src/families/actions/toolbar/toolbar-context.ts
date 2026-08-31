import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { PrimitiveElement } from "../../foundations/primitive/primitive.ts";
import type {
  ToolbarDirection,
  ToolbarItemState,
  ToolbarOrientation,
  ToolbarState,
} from "./toolbar-types.ts";

/** Registered item used by the root for DOM-order roving focus. */
export interface ToolbarItemRegistration {
  readonly value: () => string;
  readonly disabled: ComputedRef<boolean>;
  readonly element: Readonly<ShallowRef<PrimitiveElement | null>>;
}

/** Keyboard navigation request made by a Toolbar item. */
export type ToolbarNavigationIntent = "first" | "last" | "next" | "previous";

/** Shared state and actions for Toolbar compound items. */
export interface ToolbarContextValue {
  readonly activeValue: Readonly<ShallowRef<string | null>>;
  readonly dir: ComputedRef<ToolbarDirection>;
  readonly disabled: ComputedRef<boolean>;
  readonly orientation: ComputedRef<ToolbarOrientation>;
  readonly rovingFocus: ComputedRef<boolean>;
  readonly state: ComputedRef<ToolbarState>;
  readonly getItemState: (disabled: boolean) => ToolbarItemState;
  readonly moveFocus: (
    fromValue: string,
    intent: ToolbarNavigationIntent,
    options?: FocusOptions,
  ) => boolean;
  readonly registerItem: (item: ToolbarItemRegistration) => () => void;
  readonly requestItemPress: (value: string, nativeEvent: MouseEvent) => void;
  readonly setActiveValue: (value: string) => void;
  readonly syncActiveValue: (item?: ToolbarItemRegistration) => void;
}

export const toolbarContext = createContext<ToolbarContextValue>("Toolbar");
