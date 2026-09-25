/** What changed the active page. */
export type PagerChangeReason = "api" | "keyboard" | "scroll" | "tab";

/** State exposed to Pager slots and instances. */
export interface PagerSlotState<PageId extends string> {
  /** Page ids in order. */
  readonly pages: readonly PageId[];

  /** Active page. */
  readonly active: PageId;

  /** Zero-based index of the active page. */
  readonly index: number;

  /** Number of pages. */
  readonly count: number;
}

/** Public instance API of Pager. */
export interface PagerExpose<PageId extends string> extends PagerSlotState<PageId> {
  /** Scroll to a page and make it active. */
  readonly goTo: (page: PageId) => boolean;

  /** Go to the next page (no wrap); returns whether it moved. */
  readonly next: () => boolean;

  /** Go to the previous page (no wrap); returns whether it moved. */
  readonly previous: () => boolean;
}
