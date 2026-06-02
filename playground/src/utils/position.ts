/**
 * Convert a JavaScript string offset to line/column (1-based for Monaco).
 *
 * The loop uses `charCodeAt` instead of slicing/splitting so diagnostics can map
 * many offsets without allocating one substring or array per lookup. This helper
 * is intentionally simple because playground diagnostics call it on every
 * reported issue.
 */
export function offsetToLineColumn(
  source: string,
  offset: number,
): { line: number; column: number } {
  const end = Math.max(0, Math.min(offset, source.length));
  let line = 1;
  let column = 1;

  for (let index = 0; index < end; index++) {
    if (source.charCodeAt(index) === 10) {
      line++;
      column = 1;
    } else {
      column++;
    }
  }

  return { line, column };
}
