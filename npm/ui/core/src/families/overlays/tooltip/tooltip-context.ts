import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../../context.ts";
import type { TooltipState } from "./tooltip-types.ts";

/** Shared state and element registry for the Tooltip compound components. */
export interface TooltipContextValue {
  readonly id: ComputedRef<string>;
  readonly triggerId: ComputedRef<string>;
  readonly contentId: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly state: ComputedRef<TooltipState>;
  readonly delayDuration: ComputedRef<number>;
  readonly skipDelayDuration: ComputedRef<number>;
  readonly triggerElement: ShallowRef<HTMLButtonElement | null>;
  readonly contentElement: ShallowRef<HTMLDivElement | null>;
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;
  readonly openTooltip: (event?: Event | null) => boolean;
  readonly close: (event?: Event | null) => boolean;
  readonly scheduleOpen: (event?: Event | null) => boolean;
  readonly cancelOpen: () => boolean;
}

export const tooltipContext = createContext<TooltipContextValue>("Tooltip");
