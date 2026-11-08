import assert from "node:assert/strict";
import { test } from "vitest";

import { createDiffLines, createUnifiedRows } from "./diff-algorithm.js";

function sequences(values: readonly string[], maximumLength: number): string[][] {
  const result: string[][] = [[]];
  for (let length = 1; length <= maximumLength; length += 1) {
    const previous = result.filter((sequence) => sequence.length === length - 1);
    result.push(...previous.flatMap((sequence) => values.map((value) => [...sequence, value])));
  }
  return result;
}

test("reconstructs both inputs across small repeated-line combinations", () => {
  const candidates = sequences(["a", "b"], 4);

  for (const previous of candidates) {
    for (const next of candidates) {
      const lines = createDiffLines(previous.join("\n"), next.join("\n"));
      assert.deepEqual(
        lines.filter((line) => line.kind !== "added").map((line) => line.text),
        previous,
      );
      assert.deepEqual(
        lines.filter((line) => line.kind !== "removed").map((line) => line.text),
        next,
      );
      assert.deepEqual(
        lines.flatMap((line) => (line.oldLine === undefined ? [] : [line.oldLine])),
        Array.from({ length: previous.length }, (_, index) => index + 1),
      );
      assert.deepEqual(
        lines.flatMap((line) => (line.newLine === undefined ? [] : [line.newLine])),
        Array.from({ length: next.length }, (_, index) => index + 1),
      );
    }
  }
});

test("normalizes CRLF input and represents empty sides without phantom lines", () => {
  const added = createDiffLines("", "one\r\ntwo");
  assert.deepEqual(
    added.map((line) => ({ kind: line.kind, text: line.text, newLine: line.newLine })),
    [
      { kind: "added", text: "one", newLine: 1 },
      { kind: "added", text: "two", newLine: 2 },
    ],
  );

  const removed = createDiffLines("one\rtwo", "");
  assert.deepEqual(
    removed.map((line) => ({ kind: line.kind, text: line.text, oldLine: line.oldLine })),
    [
      { kind: "removed", text: "one", oldLine: 1 },
      { kind: "removed", text: "two", oldLine: 2 },
    ],
  );
});

test("merges overlapping context while keeping distant changes in separate hunks", () => {
  const previous = Array.from({ length: 12 }, (_, index) => `line ${index + 1}`);
  const next = [...previous];
  next[2] = "changed 3";
  next[4] = "changed 5";
  next[10] = "changed 11";

  const rows = createUnifiedRows(previous.join("\n"), next.join("\n"), 1);
  const headers = rows.filter((row) => row.kind === "hunk");
  assert.deepEqual(
    headers.map((header) => header.text),
    ["@@ -2,5 +2,5 @@", "@@ -10,3 +10,3 @@"],
  );
});
