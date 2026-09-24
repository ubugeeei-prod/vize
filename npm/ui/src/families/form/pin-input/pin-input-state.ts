import type { PinInputCharacters, PinInputType } from "./pin-input-types.ts";

let graphemeSegmenter: Intl.Segmenter | undefined;

/** Split text into user-perceived characters (created lazily to keep imports side-effect free). */
export function splitPinCharacters(text: string): string[] {
  graphemeSegmenter ??= new Intl.Segmenter(undefined, { granularity: "grapheme" });
  return Array.from(graphemeSegmenter.segment(text), (part) => part.segment);
}

const numericCharacter = /^\p{Nd}$/u;
const alphanumericCharacter = /^[\p{L}\p{Nd}]$/u;

/** Keep only accepted characters, folding compatibility forms such as full-width digits. */
export function sanitizePinInput(
  text: string,
  type: PinInputType,
  pattern?: RegExp,
): readonly string[] {
  const accepted: string[] = [];
  for (const character of splitPinCharacters(text)) {
    if (pattern !== undefined) {
      pattern.lastIndex = 0;
      if (pattern.test(character)) accepted.push(character);
      continue;
    }
    const test = type === "numeric" ? numericCharacter : alphanumericCharacter;
    if (!test.test(character)) continue;
    // NFKC folds full-width input ("１", "Ａ") to ASCII so codes compare predictably.
    accepted.push(character.normalize("NFKC"));
  }
  return accepted;
}

/** Normalize a length prop into a safe positive integer. */
export function normalizePinLength(length: number): number {
  return Number.isFinite(length) && length >= 1 ? Math.min(Math.floor(length), 64) : 1;
}

/** Write characters starting at `index` (clamped to the end of the value), truncated to `length`. */
export function writePinCharacters(
  value: string,
  index: number,
  characters: readonly string[],
  length: number,
): string {
  const current = splitPinCharacters(value);
  const start = Math.min(Math.max(index, 0), current.length);
  const next = [
    ...current.slice(0, start),
    ...characters,
    ...current.slice(start + characters.length),
  ];
  return next.slice(0, length).join("");
}

/** Remove the character at `index`, shifting later characters left. */
export function removePinCharacter(value: string, index: number): string {
  const current = splitPinCharacters(value);
  if (index < 0 || index >= current.length) return value;
  current.splice(index, 1);
  return current.join("");
}

/** Narrow characters to a fixed-length tuple after checking the runtime length. */
export function isPinCharacters<Length extends number>(
  characters: readonly string[],
  length: Length,
): characters is PinInputCharacters<Length> {
  return characters.length === length;
}
