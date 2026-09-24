import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { Placement } from "../positioner/positioner.ts";
import type {
  TourDirection,
  TourDismissReason,
  TourSlotState,
  TourState,
  TourStepDefinition,
  TourTargetState,
} from "./tour-types.ts";

/** Shared state and actions for the Tour compound components. */
export interface TourContextValue {
  readonly id: ComputedRef<string>;
  readonly contentId: ComputedRef<string>;
  readonly titleId: ComputedRef<string>;
  readonly descriptionId: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<TourState>;
  readonly steps: ComputedRef<readonly TourStepDefinition[]>;
  readonly step: ComputedRef<TourStepDefinition | null>;
  readonly index: ComputedRef<number>;
  readonly total: ComputedRef<number>;
  readonly first: ComputedRef<boolean>;
  readonly last: ComputedRef<boolean>;
  readonly placement: ComputedRef<Placement>;
  readonly dir: ComputedRef<TourDirection>;
  readonly keyboardNavigation: ComputedRef<boolean>;
  readonly target: Readonly<ShallowRef<Element | null>>;
  readonly targetState: ComputedRef<TourTargetState>;
  readonly slotState: ComputedRef<TourSlotState>;
  readonly next: (event?: Event | null) => boolean;
  readonly previous: (event?: Event | null) => boolean;
  readonly dismiss: (reason?: TourDismissReason, event?: Event | null) => boolean;
  readonly indexOf: (value: string) => number;
}

export const tourContext = createContext<TourContextValue>("Tour");
