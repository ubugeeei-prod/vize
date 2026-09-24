/** Accessible, unstyled table of contents that tracks the section in view. */
export { default as Toc, default as TocRoot } from "./toc-root.vue";
export { default as TocItem } from "./toc-item.vue";
export { default as TocLink } from "./toc-link.vue";
export { default as TocList } from "./toc-list.vue";
export { collectTocEntries } from "./toc-collect.ts";
export type {
  TocCollectOptions,
  TocEntry,
  TocLinkSlotState,
  TocRootExpose,
  TocScrollBehavior,
  TocSlotState,
  TocState,
} from "./toc-types.ts";
