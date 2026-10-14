import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";

/** Type-erased NavigationRail state shared with items and the toggle. */
export interface NavigationRailContextValue {
  /** Id of the navigation landmark, referenced by the toggle's `aria-controls`. */
  readonly id: ComputedRef<string>;
  readonly active: ComputedRef<string | undefined>;
  /** Whether destinations show their labels beside the icons. */
  readonly expanded: ComputedRef<boolean>;
  readonly select: (value: string, event: MouseEvent) => void;
  readonly setExpanded: (value: boolean) => void;
}

/** Typed context shared by NavigationRail, its items, and its toggle. */
export const navigationRailContext = createContext<NavigationRailContextValue>("NavigationRail");
