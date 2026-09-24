import type { HotspotShape } from "./hotspot-geometry.ts";

/**
 * One marker declared on HotspotRoot. `Data` carries consumer payload that is
 * returned, fully typed, through slots and emits.
 */
export interface HotspotDefinition<Data = undefined> {
  /** Stable id used by `v-model:active`. */
  readonly id: string;

  /** Horizontal position in percent of the image width. */
  readonly x: number;

  /** Vertical position in percent of the image height. */
  readonly y: number;

  /** Accessible marker name. */
  readonly label: string;

  /** Consumer payload. */
  readonly data?: Data;

  /** Remove the marker from interaction. */
  readonly disabled?: boolean;
}

/** Marker state mirrored through `data-state`. */
export type HotspotMarkerState = "closed" | "disabled" | "open";

/** Why the active marker changed. */
export type HotspotActiveChangeSource = "api" | "area" | "dismiss" | "marker";

/** State exposed to the HotspotRoot default slot. */
export interface HotspotSlotState<Data = undefined> {
  /** Declared markers, in input order. */
  readonly hotspots: readonly HotspotDefinition<Data>[];

  /** Id of the most recently opened marker, or `null`. */
  readonly active: string | null;

  /** Ids of every open marker. */
  readonly openIds: readonly string[];
}

/** State exposed to HotspotMarker and HotspotContent slots. */
export interface HotspotMarkerSlotState {
  /** Marker id. */
  readonly id: string;

  /** Whether the marker content is open. */
  readonly open: boolean;

  /** Whether the marker is disabled. */
  readonly disabled: boolean;

  /** Stable state token. */
  readonly state: HotspotMarkerState;
}

/** State exposed to HotspotArea slots. */
export interface HotspotAreaSlotState {
  /** Area id. */
  readonly id: string;

  /** Whether this area is the active hotspot. */
  readonly active: boolean;
}

/** Public instance exposed by HotspotRoot. */
export interface HotspotRootExpose {
  /** Rendered root element. */
  readonly element: HTMLDivElement | null;

  /** Most recently opened marker, or `null`. */
  readonly active: string | null;

  /** Ids of every open marker. */
  readonly openIds: readonly string[];

  /** Open one marker (closing others unless `exclusive` is false). Reports whether state changed. */
  readonly open: (id: string) => boolean;

  /** Close one marker, or every marker without an id. Reports whether state changed. */
  readonly close: (id?: string) => boolean;

  /** Ids of registered HotspotAreas whose shape contains an image-space point (percent). */
  readonly hitTest: (x: number, y: number) => readonly string[];
}

/** Public instance exposed by HotspotMarker. */
export interface HotspotMarkerExpose extends HotspotMarkerSlotState {
  /** Rendered marker wrapper. */
  readonly element: HTMLDivElement | null;

  /** Focus the marker button. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by HotspotImage. */
export interface HotspotImageExpose {
  /** Rendered native image. */
  readonly element: HTMLImageElement | null;
}

/** Public instance exposed by HotspotArea. */
export interface HotspotAreaExpose extends HotspotAreaSlotState {
  /** Rendered SVG overlay. */
  readonly element: SVGSVGElement | null;

  /** Area shape. */
  readonly shape: HotspotShape;
}
