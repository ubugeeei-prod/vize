import type { CommandPaletteFilter } from "./command-palette-types.ts";

function normalize(value: string): string {
  return value.normalize("NFKD").replaceAll(/\p{M}/gu, "").toLocaleLowerCase().trim();
}

function subsequenceScore(text: string, search: string): number {
  let position = 0;
  let gaps = 0;
  for (const character of search) {
    const found = text.indexOf(character, position);
    if (found === -1) return 0;
    gaps += found - position;
    position = found + character.length;
  }
  return 1 / (1 + gaps);
}

function scoreText(text: string, search: string): number {
  if (text === search) return 4;
  if (text.startsWith(search)) return 3;
  if (text.split(/[\s\-_/.:]+/u).some((word) => word.startsWith(search))) return 2.5;
  if (text.includes(search)) return 2;
  return subsequenceScore(text, search);
}

/**
 * Default palette scorer: exact, prefix, word-prefix, substring, then an
 * in-order subsequence ("fuzzy") match, case- and diacritic-insensitive.
 * Keywords score like the label, slightly discounted.
 */
export const defaultCommandPaletteFilter: CommandPaletteFilter = (text, search, keywords) => {
  const query = normalize(search);
  if (query === "") return 1;
  const label = scoreText(normalize(text), query);
  let best = label;
  for (const keyword of keywords) best = Math.max(best, scoreText(normalize(keyword), query) * 0.9);
  return best;
};

/** Default English result announcement. */
export function defaultCommandPaletteResultsLabel(count: number): string {
  return count === 1 ? "1 result" : `${count} results`;
}
