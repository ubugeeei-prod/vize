import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  MarqueeDirection,
  MarqueeMeasurement,
  MarqueeMessages,
  MarqueeOrientation,
  MarqueeSlotState,
  MarqueeState,
} from "./marquee-types.ts";

/** Shared state for the Marquee compound parts. */
export interface MarqueeContextValue {
  readonly id: ComputedRef<string>;
  readonly element: Readonly<ShallowRef<HTMLDivElement | null>>;
  readonly state: ComputedRef<MarqueeState>;
  readonly direction: ComputedRef<MarqueeDirection>;
  readonly orientation: ComputedRef<MarqueeOrientation>;
  readonly copies: ComputedRef<number>;
  /** Whether the button should offer "pause": intent on and not blocked by reduced motion. */
  readonly effectivePlaying: ComputedRef<boolean>;
  readonly messages: ComputedRef<MarqueeMessages>;
  readonly slotState: ComputedRef<MarqueeSlotState>;
  readonly setMeasurement: (measurement: MarqueeMeasurement) => void;
  readonly toggle: () => boolean;
}

export const marqueeContext = createContext<MarqueeContextValue>("Marquee");
