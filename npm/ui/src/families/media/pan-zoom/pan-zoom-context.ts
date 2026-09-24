import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  PanZoomChangeSource,
  PanZoomMessages,
  PanZoomPoint,
  PanZoomSlotState,
  PanZoomState,
  PanZoomTransform,
  PanZoomWheelMode,
} from "./pan-zoom-types.ts";

/** Shared state and actions for the PanZoom compound parts. */
export interface PanZoomContextValue {
  readonly transform: ComputedRef<PanZoomTransform>;
  readonly state: ComputedRef<PanZoomState>;
  readonly disabled: ComputedRef<boolean>;
  readonly slotState: ComputedRef<PanZoomSlotState>;
  readonly messages: ComputedRef<Required<Omit<PanZoomMessages, "zoomLevel">>>;
  readonly statusText: ComputedRef<string>;
  readonly wheelMode: ComputedRef<PanZoomWheelMode>;
  readonly doubleClickZoom: ComputedRef<boolean>;
  readonly panStep: ComputedRef<number>;
  readonly minScale: ComputedRef<number>;
  readonly setViewportElement: (element: HTMLElement | null) => void;
  readonly setContentElement: (element: HTMLElement | null) => void;
  readonly setTransform: (transform: PanZoomTransform, source: PanZoomChangeSource) => boolean;
  readonly zoomTo: (
    scale: number,
    point: PanZoomPoint | undefined,
    source: PanZoomChangeSource,
  ) => boolean;
  readonly zoomBy: (
    steps: number,
    point: PanZoomPoint | undefined,
    source: PanZoomChangeSource,
  ) => boolean;
  readonly panBy: (dx: number, dy: number, source: PanZoomChangeSource) => boolean;
  readonly reset: (source: PanZoomChangeSource) => boolean;
  readonly fit: (source: PanZoomChangeSource) => boolean;
  readonly beginGesture: (
    state: Exclude<PanZoomState, "disabled">,
    source: PanZoomChangeSource,
  ) => void;
  readonly endGesture: (source: PanZoomChangeSource) => void;
  readonly limits: ComputedRef<{ readonly minScale: number; readonly maxScale: number }>;
}

export const panZoomContext = createContext<PanZoomContextValue>("PanZoom");
