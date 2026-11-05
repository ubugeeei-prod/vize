/** Exact byte geometry shared by mapper and virtual packet relocation. */
import assert from "node:assert/strict";
export type Edit = { start: number; end: number; replacement: Buffer };
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
export function point(value: number, edits: Edit[]) {
  integer(value);
  let delta = 0;
  for (const edit of edits) {
    if (value >= edit.end) delta += edit.replacement.length - (edit.end - edit.start);
    else {
      assert.ok(value <= edit.start, "coordinate enters a certified replacement");
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
