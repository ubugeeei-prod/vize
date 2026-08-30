/** Accessible, unstyled Pagination primitive with controlled page state and SSR-stable ids. */
export { default as Pagination, default as PaginationRoot } from "./pagination.vue";
export { default as PaginationEllipsis } from "./pagination-ellipsis.vue";
export { default as PaginationItem } from "./pagination-item.vue";
export { default as PaginationList } from "./pagination-list.vue";
export { default as PaginationNext } from "./pagination-next.vue";
export { default as PaginationPage } from "./pagination-page.vue";
export { default as PaginationPrevious } from "./pagination-previous.vue";
export type {
  PaginationControlExpose,
  PaginationControlSlotState,
  PaginationControlState,
  PaginationEllipsisExpose,
  PaginationEllipsisPosition,
  PaginationEllipsisSlotState,
  PaginationItemExpose,
  PaginationItemSlotState,
  PaginationListExpose,
  PaginationListSlotState,
  PaginationPageExpose,
  PaginationPageSlotState,
  PaginationPageState,
  PaginationRootExpose,
  PaginationRootProps,
  PaginationSlotState,
  PaginationState,
} from "./pagination-types.ts";
export type {
  PaginationRangeEllipsis,
  PaginationRangeItem,
  PaginationRangeOptions,
  PaginationRangePage,
} from "./pagination-range.ts";
export {
  createPaginationRange,
  getPaginationPageIdSegment,
  normalizePaginationPage,
  normalizePaginationPageCount,
  toPaginationPageInRange,
} from "./pagination-range.ts";
