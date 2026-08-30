/** One numbered page rendered by the Pagination range helper. */
export interface PaginationRangePage {
  readonly type: "page";
  readonly key: `page-${number}`;
  readonly page: number;
}

/** A non-interactive gap between two numbered page runs. */
export interface PaginationRangeEllipsis {
  readonly type: "ellipsis";
  readonly key: "ellipsis-start" | "ellipsis-end";
  readonly position: "start" | "end";
}

/** Item emitted by the Pagination range helper. */
export type PaginationRangeItem = PaginationRangePage | PaginationRangeEllipsis;

export interface PaginationRangeOptions {
  /** Current page, clamped into `1...pageCount`. */
  readonly page: number;

  /** Total page count. Values below one resolve to one page. */
  readonly pageCount: number;

  /** Number of pages kept on each side of the current page. */
  readonly siblingCount?: number;

  /** Number of pages always kept at each boundary. */
  readonly boundaryCount?: number;
}

/** Normalize an arbitrary page count into a finite positive integer. */
export function normalizePaginationPageCount(value: number): number {
  if (!Number.isFinite(value)) return 1;
  const integer = Math.trunc(value);
  return integer < 1 ? 1 : integer;
}

/** Clamp a page value into the current page-count range. */
export function normalizePaginationPage(
  value: number | null | undefined,
  pageCount: number,
): number {
  const normalizedCount = normalizePaginationPageCount(pageCount);
  if (value == null || !Number.isFinite(value)) return 1;
  const integer = Math.trunc(value);
  if (integer < 1) return 1;
  return integer > normalizedCount ? normalizedCount : integer;
}

/** Return an in-range page number, or `null` when a control points outside the range. */
export function toPaginationPageInRange(value: number, pageCount: number): number | null {
  if (!Number.isFinite(value)) return null;
  const integer = Math.trunc(value);
  if (integer < 1 || integer > normalizePaginationPageCount(pageCount)) return null;
  return integer;
}

/** Return a deterministic page id segment that never collides with valid page ids. */
export function getPaginationPageIdSegment(value: number, pageCount: number): string {
  if (!Number.isFinite(value)) return `page-invalid-${String(value).toLowerCase()}`;
  const integer = Math.trunc(value);
  if (integer < 1) return `page-before-${Math.abs(integer)}`;
  if (integer > normalizePaginationPageCount(pageCount)) return `page-after-${integer}`;
  return `page-${integer}`;
}

/** Create a deterministic compact page range with one-page gaps expanded. */
export function createPaginationRange(options: PaginationRangeOptions): PaginationRangeItem[] {
  const pageCount = normalizePaginationPageCount(options.pageCount);
  const page = normalizePaginationPage(options.page, pageCount);
  const siblingCount = normalizeMargin(options.siblingCount ?? 1);
  const boundaryCount = normalizeMargin(options.boundaryCount ?? 1);
  const visible = new Set<number>();

  addPages(visible, 1, boundaryCount, pageCount);
  addPages(visible, pageCount - boundaryCount + 1, pageCount, pageCount);
  addPages(visible, page - siblingCount, page + siblingCount, pageCount);

  const pages = [...visible].sort((left, right) => left - right);
  const range: PaginationRangeItem[] = [];
  let previous: number | null = null;

  for (const next of pages) {
    if (previous !== null) {
      const gap = next - previous;
      if (gap === 2) range.push(createPage(previous + 1));
      else if (gap > 2) range.push(createEllipsis(previous >= page ? "end" : "start"));
    }
    range.push(createPage(next));
    previous = next;
  }

  return range;
}

function normalizeMargin(value: number): number {
  if (!Number.isFinite(value)) return 0;
  const integer = Math.trunc(value);
  return integer < 0 ? 0 : integer;
}

function addPages(target: Set<number>, start: number, end: number, pageCount: number): void {
  const first = Math.max(1, start);
  const last = Math.min(pageCount, end);
  for (let page = first; page <= last; page++) target.add(page);
}

function createPage(page: number): PaginationRangePage {
  return { key: `page-${page}`, page, type: "page" };
}

function createEllipsis(position: "start" | "end"): PaginationRangeEllipsis {
  return { key: `ellipsis-${position}`, position, type: "ellipsis" };
}
