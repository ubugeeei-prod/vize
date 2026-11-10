import assert from "node:assert/strict";
import { test } from "node:test";
import { dedentEdits } from "../../tools/support/compat/content-mapper/setup-dedent.ts";

const region = (body: string[], comment: string) =>
  Buffer.from(
    `head\n  // User setup code\n${body.map((line) => `${line}\n`).join("")}  // @vize-map: ${comment}\ntail\n`,
  );
const lines = ["", "const a = `", "x`", "  if (a) {}"];
const before = region(
  lines.map((line) => `  ${line}`),
  "26:62 -> 0:20",
);
const after = region(lines, "26:54 -> 0:20");

void test("each authored setup line loses exactly its two-space prefix", () => {
  const edits = dedentEdits(before, after);
  assert.equal(edits.length, lines.length);
  const parts: Buffer[] = [];
  let cursor = 0;
  for (const edit of edits) {
    assert.equal(edit.end - edit.start, 2);
    parts.push(before.subarray(cursor, edit.start));
    cursor = edit.end;
  }
  parts.push(before.subarray(cursor));
  assert.equal(Buffer.concat(parts).toString(), after.toString().replace("26:54", "26:62"));
  assert.deepEqual(
    edits.map((edit) => edit.frame),
    edits.map((edit) => edit.start),
  );
});

void test("any other byte change in the setup region is refused", () => {
  const deeper = region(
    lines.map((line, index) => (index === 1 ? `    ${line}` : `  ${line}`)),
    "26:64 -> 0:20",
  );
  assert.throws(() => dedentEdits(deeper, after), /two-space/u);
  const changed = region([...lines.slice(0, 3), "  if (b) {}"], "26:54 -> 0:20");
  assert.throws(() => dedentEdits(before, changed), /two-space/u);
  assert.throws(() => dedentEdits(before, Buffer.from("no setup\n")), /inventory/u);
});
