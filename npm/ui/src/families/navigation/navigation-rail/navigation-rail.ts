/**
 * Side navigation rail: a labelled navigation landmark of top-level
 * destinations whose current one carries `aria-current="page"`, with an
 * optional expanded state that shows labels beside the icons.
 */
export { default as NavigationRail } from "./navigation-rail.vue";
/** Destination link or button inside NavigationRail. */
export { default as NavigationRailItem } from "./navigation-rail-item.vue";
/** Button that expands or collapses the rail. */
export { default as NavigationRailToggle } from "./navigation-rail-toggle.vue";
export type { NavigationRailExpose, NavigationRailSlotState } from "./navigation-rail-types.ts";
