import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { HotspotShape } from "./hotspot-geometry.ts";
import type { HotspotActiveChangeSource } from "./hotspot-types.ts";

/** Registered marker geometry used for spatial keyboard navigation and hit testing. */
export interface HotspotRegistration {
  readonly id: string;
  readonly x: () => number;
  readonly y: () => number;
  readonly disabled: () => boolean;
  readonly focus: () => void;
}

/** Shared state and actions for the Hotspot compound parts. */
export interface HotspotContextValue {
  readonly id: ComputedRef<string>;
  readonly active: ComputedRef<string | null>;
  readonly isOpen: (id: string) => boolean;
  readonly setOpen: (id: string, open: boolean, source: HotspotActiveChangeSource) => boolean;
  readonly registerMarker: (registration: HotspotRegistration) => () => void;
  readonly registerArea: (id: string, shape: () => HotspotShape) => () => void;
  readonly focusNeighbor: (fromId: string, key: string) => boolean;
  readonly getMarkerId: (id: string) => string;
}

export const hotspotContext = createContext<HotspotContextValue>("Hotspot");

/** Per-marker values shared with HotspotContent. */
export interface HotspotMarkerContextValue {
  readonly id: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
}

export const hotspotMarkerContext = createContext<HotspotMarkerContextValue>("HotspotMarker");
