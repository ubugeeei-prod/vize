import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";

/** One registered action, in declaration order. */
export interface ActionSheetItemRegistration {
  readonly id: string;
  readonly disabled: () => boolean;
  readonly textValue: () => string;
  readonly element: () => HTMLElement | null;
}

/** Shared roving-focus state for ActionSheetMenu items. */
export interface ActionSheetMenuContextValue {
  readonly activeId: ComputedRef<string | undefined>;
  readonly items: Readonly<ShallowRef<readonly ActionSheetItemRegistration[]>>;
  readonly register: (item: ActionSheetItemRegistration) => () => void;
  readonly setActive: (id: string) => void;
  readonly close: (event: Event) => void;
}

/** Typed context shared by ActionSheetMenu and ActionSheetItem. */
export const actionSheetMenuContext = createContext<ActionSheetMenuContextValue>("ActionSheetMenu");
