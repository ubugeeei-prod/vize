import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { CollapsibleState } from "./collapsible-types.ts";

/** Shared state and ids for the Collapsible compound components. */
export interface CollapsibleContextValue {
  readonly id: ComputedRef<string>;
  readonly triggerId: ComputedRef<string>;
  readonly contentId: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly state: ComputedRef<CollapsibleState>;
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;
  readonly expand: (event?: Event | null) => boolean;
  readonly collapse: (event?: Event | null) => boolean;
  readonly toggle: (event?: Event | null) => boolean;
}

export const collapsibleContext = createContext<CollapsibleContextValue>("Collapsible");
