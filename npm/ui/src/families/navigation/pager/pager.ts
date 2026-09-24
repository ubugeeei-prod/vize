/** Swipeable, scroll-snap pager with typed page ids and a segmented tab bar (APG tabs). */
export { default as Pager } from "./pager.vue";
/** `role="tablist"` segmented control for the pages. */
export { default as PagerTabList } from "./pager-tab-list.vue";
/** One segment (`role="tab"`) selecting a page. */
export { default as PagerTab } from "./pager-tab.vue";
/** Horizontal scroll-snap container; user swipes settle on the nearest page. */
export { default as PagerViewport } from "./pager-viewport.vue";
/** One page (`role="tabpanel"`), inert while off-screen. */
export { default as PagerPage } from "./pager-page.vue";
export { pagerIds } from "./pager-context.ts";
export type { PagerChangeReason, PagerExpose, PagerSlotState } from "./pager-types.ts";
