/**
 * Accessible, unstyled GridList (WAI-ARIA APG grid pattern for interactive
 * lists): roving focus, typeahead, single/multiple selection with ranges,
 * list or 2D grid layouts, and pointer + keyboard drag reordering.
 */
export { default as GridList } from "./grid-list.vue";
export { default as GridListItem } from "./grid-list-item.vue";
export type {
  GridListExpose,
  GridListItemSlotProps,
  GridListLayout,
  GridListReorderEvent,
  GridListSelectionMode,
} from "./grid-list-types.ts";
