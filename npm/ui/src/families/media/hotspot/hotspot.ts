/** Headless image hotspots: positioned markers with popover details and SVG regions. */
export { default as HotspotArea } from "./hotspot-area.vue";
export { default as HotspotContent } from "./hotspot-content.vue";
export { default as HotspotImage } from "./hotspot-image.vue";
export { default as HotspotMarker } from "./hotspot-marker.vue";
export { default as Hotspot, default as HotspotRoot } from "./hotspot-root.vue";
export {
  clampHotspotCoordinate,
  hotspotPolygonPoints,
  hotspotShapeContains,
  isPointInHotspotPolygon,
  nextHotspotInDirection,
  sanitizeHotspotHref,
} from "./hotspot-geometry.ts";
export type {
  HotspotCircleShape,
  HotspotDirection,
  HotspotNavigationItem,
  HotspotPoint,
  HotspotPolygonShape,
  HotspotRectShape,
  HotspotShape,
} from "./hotspot-geometry.ts";
export type {
  HotspotActiveChangeSource,
  HotspotAreaExpose,
  HotspotAreaSlotState,
  HotspotDefinition,
  HotspotImageExpose,
  HotspotMarkerExpose,
  HotspotMarkerSlotState,
  HotspotMarkerState,
  HotspotRootExpose,
  HotspotSlotState,
} from "./hotspot-types.ts";
