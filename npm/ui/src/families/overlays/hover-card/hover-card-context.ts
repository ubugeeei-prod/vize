import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  HoverCardOpenReason,
  HoverCardState,
  HoverCardTouchBehavior,
} from "./hover-card-types.ts";

/** Shared state, timers, and element registry for the HoverCard compound components. */
export interface HoverCardContextValue {
  readonly id: ComputedRef<string>;
  readonly triggerId: ComputedRef<string>;
  readonly contentId: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly state: ComputedRef<HoverCardState>;
  readonly reason: Readonly<ShallowRef<HoverCardOpenReason | null>>;
  readonly touchBehavior: ComputedRef<HoverCardTouchBehavior>;
  readonly longPressDelay: ComputedRef<number>;
  readonly triggerElement: ShallowRef<HTMLElement | null>;
  readonly contentElement: ShallowRef<HTMLDivElement | null>;
  readonly setOpen: (value: boolean, event?: Event | null, reason?: HoverCardOpenReason) => boolean;
  readonly scheduleOpen: (event?: Event | null, reason?: HoverCardOpenReason) => boolean;
  readonly scheduleClose: (event?: Event | null) => boolean;
  readonly cancelPending: () => boolean;
}

export const hoverCardContext = createContext<HoverCardContextValue>("HoverCard");
