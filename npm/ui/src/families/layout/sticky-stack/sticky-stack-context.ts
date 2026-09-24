import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";

/** Registration of one stacked item. */
export interface StickyStackRegistration {
  readonly id: symbol;
  readonly element: () => Element | null;
  readonly height: () => number;
  readonly enabled: () => boolean;
}

/** Shared state between `StickyStack` and its items. */
export interface StickyStackContextValue {
  readonly register: (registration: StickyStackRegistration) => () => void;
  readonly topOf: (id: symbol) => number;
  readonly total: ComputedRef<number>;
}

export const stickyStackContext = createContext<StickyStackContextValue>("StickyStack");
