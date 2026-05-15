/** Convert character offset to line/column (1-based for Monaco) */
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
