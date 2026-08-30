import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../../context.ts";
import type { PopoverState } from "./popover-types.ts";

/** Shared state and element registry for the Popover compound components. */
export interface PopoverContextValue {
  readonly id: ComputedRef<string>;
  readonly triggerId: ComputedRef<string>;
  readonly contentId: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly modal: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly state: ComputedRef<PopoverState>;
  readonly triggerElement: ShallowRef<HTMLButtonElement | null>;
  readonly contentElement: ShallowRef<HTMLDivElement | null>;
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;
  readonly openPopover: (event?: Event | null) => boolean;
  readonly close: (event?: Event | null) => boolean;
  readonly toggle: (event?: Event | null) => boolean;
}

export const popoverContext = createContext<PopoverContextValue>("Popover");
