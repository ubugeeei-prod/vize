/** Exact byte geometry shared by mapper and virtual packet relocation. */
import assert from "node:assert/strict";
/**
 * `at` places a replacement whose bytes are not unique in the current text;
 * `frame` is its start in the self-described frame of `// @vize-map` comments.
 */
export type Edit = { start: number; end: number; replacement: Buffer; at?: number; frame?: number };
export function integer(value: number) {
  assert.ok(Number.isSafeInteger(value) && value >= 0, `invalid byte coordinate: ${value}`);
}
export function bounds(start: number, length: number, limit: number) {
  integer(start);
  integer(length);
  assert.ok(start + length <= limit, `range ${start}:${start + length} exceeds ${limit}`);
}
export function overlaps(start: number, length: number, edit: Edit) {
  return start < edit.end && edit.start < start + length;
}
export function point(value: number, edits: Edit[], selfDescribed = false) {
  integer(value);
  let delta = 0;
  for (const edit of edits) {
    const start = selfDescribed ? (edit.frame ?? edit.start) : edit.start;
    if (value >= start + edit.end - edit.start)
      delta += edit.replacement.length - (edit.end - edit.start);
    else {
      assert.ok(value <= start, "coordinate enters a certified replacement");
      break;
    }
  }
  return value + delta;
}
export function range(start: number, length: number, edits: Edit[], limit: number) {
  bounds(start, length, limit);
  assert.ok(
    edits.every((edit) => !overlaps(start, length, edit)),
    "authored projection enters a certified replacement",
  );
  const relocated = point(start, edits);
  return [relocated, point(start + length, edits) - relocated];
}
export function rangeIncludingInsertions(
  start: number,
  length: number,
  edits: Edit[],
  limit: number,
) {
  bounds(start, length, limit);
  assert.ok(
    edits.every((edit) => edit.start === edit.end || !overlaps(start, length, edit)),
    "diagnostic projection enters a certified replacement",
  );
  const relocated = point(start, edits);
  return [relocated, point(start + length, edits) - relocated];
}
export function bufferOccurrences(text: Buffer, needle: Buffer) {
  let count = 0;
  for (let offset = text.indexOf(needle); offset >= 0; offset = text.indexOf(needle, offset + 1))
    count++;
  return count;
}
export function comments(text: string, generatedRanges: [number, number][]) {
  return [...text.matchAll(/^([ \t]*)\/\/ @vize-map: (\d+):(\d+) -> (\d+):(\d+)\r?$/gmu)]
    .map((match) => ({
      text: match[0],
      values: match.slice(2, 6).map(Number),
      indent: match[1],
      start: Buffer.byteLength(text.slice(0, match.index)),
    }))
    .filter(
      (comment) =>
        !generatedRanges.some(
          ([start, length]) =>
            start < comment.start + Buffer.byteLength(comment.text) &&
            comment.start < start + length,
        ),
    );
}
