/** Certify #12481: authored setup lines lose only the two spaces the old projection prefixed. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import type { Edit } from "./projection-byte-relocation.ts";

const producer = readFileSync(
  new URL("../../../../crates/vize_canon/src/virtual_ts/generator.rs", import.meta.url),
);
assert.equal(producer.length, 38188, "certified verbatim setup producer size");
assert.equal(
  createHash("sha256").update(producer).digest("hex"),
  "87839dacc6072fc16e33f4bc1d236a27de5b6659af31a16d5bae802f7dfbaf60",
  "certified verbatim setup producer source",
);

const marker = Buffer.from("  // User setup code\n");
const terminator = /^  \/\/ @vize-map: (\d+):\d+ -> \d+:\d+\r?$/u;

function offsets(text: Buffer) {
  const found: number[] = [];
  for (let at = text.indexOf(marker); at >= 0; at = text.indexOf(marker, at + 1)) found.push(at);
  return found;
}

function lineEnd(text: Buffer, start: number) {
  const end = text.indexOf(10, start);
  assert.ok(end >= 0, "a user setup region must end with its map comment");
  return end;
}

/** One two-byte deletion per authored setup line; every other byte must match exactly. */
export function dedentEdits(before: Buffer, after: Buffer): (Edit & { label: string })[] {
  const starts = offsets(before);
  const currents = offsets(after);
  assert.equal(currents.length, starts.length, "user setup region inventory changed");
  const edits: (Edit & { label: string })[] = [];
  for (const [index, start] of starts.entries()) {
    let left = start + marker.length;
    const first = edits.length;
    let right = currents[index]! + marker.length;
    for (;;) {
      const leftEnd = lineEnd(before, left);
      const rightEnd = lineEnd(after, right);
      const old = before.subarray(left, leftEnd);
      const current = after.subarray(right, rightEnd);
      const described = terminator.exec(old.toString());
      if (described && terminator.test(current.toString())) {
        // The comment counts from where the setup body begins in its own frame.
        const shift = start + marker.length - Number(described[1]);
        for (const edit of edits.slice(first)) edit.frame = edit.start - shift;
        break;
      }
      if (!old.equals(current)) {
        assert.ok(
          old.length === current.length + 2 && old[0] === 32 && old[1] === 32,
          "only the two-space setup indentation may change",
        );
        assert.ok(
          old.subarray(2).equals(current),
          "only the two-space setup indentation may change",
        );
        edits.push({
          start: left,
          end: left + 2,
          replacement: Buffer.alloc(0),
          at: right,
          label: "verbatim setup line",
        });
      }
      left = leftEnd + 1;
      right = rightEnd + 1;
    }
  }
  return edits;
}
