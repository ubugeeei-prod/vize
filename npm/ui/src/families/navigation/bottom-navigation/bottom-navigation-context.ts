import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";

/** Type-erased BottomNavigation state shared with items. */
export interface BottomNavigationContextValue {
  readonly active: ComputedRef<string | undefined>;
  readonly select: (value: string, event: MouseEvent) => void;
}

/** Typed context shared by BottomNavigation and its items. */
export const bottomNavigationContext =
  createContext<BottomNavigationContextValue>("BottomNavigation");
