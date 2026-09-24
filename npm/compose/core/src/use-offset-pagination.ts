import { computed, isRef, shallowRef, toValue, watch } from "vue";
import type { ComputedRef, MaybeRef, MaybeRefOrGetter, Ref, WritableComputedRef } from "vue";

/** Page information passed to the {@link useOffsetPagination} callbacks. */
export interface OffsetPaginationInfo {
  /** One-based current page. */
  readonly currentPage: number;

  /** Items per page. */
  readonly currentPageSize: number;

  /** Number of pages (at least one). */
  readonly pageCount: number;
}

/** Options for {@link useOffsetPagination}. */
export interface UseOffsetPaginationOptions {
  /**
   * Total number of items. Reactive.
   *
   * @default Number.POSITIVE_INFINITY
   */
  readonly total?: MaybeRefOrGetter<number>;

  /**
   * One-based current page. Pass a writable ref for two-way binding.
   *
   * @default 1
   */
  readonly page?: MaybeRef<number>;

  /**
   * Items per page. Pass a writable ref for two-way binding.
   *
   * @default 10
   */
  readonly pageSize?: MaybeRef<number>;

  /**
   * Called after the current page changes.
   *
   * @default undefined
   */
  readonly onPageChange?: (info: OffsetPaginationInfo) => void;

  /**
   * Called after the page size changes.
   *
   * @default undefined
   */
  readonly onPageSizeChange?: (info: OffsetPaginationInfo) => void;

  /**
   * Called after the page count changes.
   *
   * @default undefined
   */
  readonly onPageCountChange?: (info: OffsetPaginationInfo) => void;
}

/** Pagination state and controls returned by {@link useOffsetPagination}. */
export interface OffsetPagination {
  /** One-based current page, always within `[1, pageCount]`. Writable. */
  readonly currentPage: WritableComputedRef<number>;

  /** Items per page, at least one. Writable. */
  readonly currentPageSize: WritableComputedRef<number>;

  /** Number of pages (at least one). */
  readonly pageCount: ComputedRef<number>;

  /** Zero-based offset of the first item of the current page. */
  readonly offset: ComputedRef<number>;

  /** Whether the current page is the first. */
  readonly isFirstPage: ComputedRef<boolean>;

  /** Whether the current page is the last. */
  readonly isLastPage: ComputedRef<boolean>;

  /** Go to the previous page (no-op on the first). */
  readonly prev: () => void;

  /** Go to the next page (no-op on the last). */
  readonly next: () => void;
}

function toWritable(value: MaybeRef<number> | undefined, fallback: number): Ref<number> {
  return isRef(value) ? value : shallowRef(value ?? fallback);
}

/**
 * Offset-based pagination state.
 *
 * The page is clamped into `[1, pageCount]` on every read and write, so
 * shrinking `total` or growing `pageSize` never leaves the cursor past the
 * end. Writable `page`/`pageSize` refs are updated in place for two-way
 * binding. Callbacks fire after the corresponding value settles.
 * Synchronous state: SSR-safe; watchers follow the owning scope.
 *
 * @example
 * ```ts
 * const { currentPage, pageCount, next, offset } = useOffsetPagination({ total, pageSize: 20 });
 * ```
 *
 * @param options Total, page, page size, and change callbacks.
 * @default options {}
 * @returns Pagination state and controls.
 */
export function useOffsetPagination(options: UseOffsetPaginationOptions = {}): OffsetPagination {
  const page = toWritable(options.page, 1);
  const size = toWritable(options.pageSize, 10);

  const currentPageSize = computed<number>({
    get: () => Math.max(1, Math.trunc(size.value)),
    set: (value) => {
      size.value = Math.max(1, Math.trunc(value));
    },
  });
  const pageCount = computed(() => {
    const total = toValue(options.total ?? Number.POSITIVE_INFINITY);
    return Math.max(1, Math.ceil(Math.max(0, total) / currentPageSize.value));
  });
  const clampPage = (value: number): number =>
    Math.min(pageCount.value, Math.max(1, Number.isNaN(value) ? 1 : Math.trunc(value)));
  const currentPage = computed<number>({
    get: () => clampPage(page.value),
    set: (value) => {
      page.value = clampPage(value);
    },
  });
  const offset = computed(() => (currentPage.value - 1) * currentPageSize.value);
  const isFirstPage = computed(() => currentPage.value === 1);
  const isLastPage = computed(() => currentPage.value === pageCount.value);

  const info = (): OffsetPaginationInfo => ({
    currentPage: currentPage.value,
    currentPageSize: currentPageSize.value,
    pageCount: pageCount.value,
  });
  if (options.onPageChange !== undefined) {
    const callback = options.onPageChange;
    watch(currentPage, () => callback(info()));
  }
  if (options.onPageSizeChange !== undefined) {
    const callback = options.onPageSizeChange;
    watch(currentPageSize, () => callback(info()));
  }
  if (options.onPageCountChange !== undefined) {
    const callback = options.onPageCountChange;
    watch(pageCount, () => callback(info()));
  }

  return {
    currentPage,
    currentPageSize,
    pageCount,
    offset,
    isFirstPage,
    isLastPage,
    prev: () => {
      currentPage.value -= 1;
    },
    next: () => {
      currentPage.value += 1;
    },
  };
}
