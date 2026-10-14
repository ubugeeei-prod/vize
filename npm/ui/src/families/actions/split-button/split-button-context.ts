import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";

/** Coordination between SplitButtonRoot and its primary action. */
export interface SplitButtonContextValue {
  /** Whether the whole split button is disabled. */
  readonly disabled: ComputedRef<boolean>;

  /**
   * Open the menu on behalf of the primary action. Focus returns to `origin`
   * instead of the trigger when the menu closes.
   */
  readonly openFromAction: (origin: HTMLElement | null, event: Event) => boolean;
}

export const splitButtonContext = createContext<SplitButtonContextValue>("SplitButton");
