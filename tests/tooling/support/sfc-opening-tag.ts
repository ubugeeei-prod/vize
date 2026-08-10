/** Find an SFC opening-tag boundary without treating quoted `>` as syntax. */
export function findSfcOpeningTagEnd(source: string, offset: number): number {
  let quote: '"' | "'" | null = null;
  for (let index = offset; index < source.length; index += 1) {
    const character = source[index];
    if (quote != null) {
      if (character === quote) quote = null;
    } else if (character === '"' || character === "'") {
      quote = character;
    } else if (character === ">") {
      return index;
    }
  }
  return -1;
}
