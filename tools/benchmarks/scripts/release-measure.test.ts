import assert from "node:assert/strict";
import { test } from "node:test";
import { foldRuns, movedRows, renderReport, type Side } from "./release-measure.ts";

const side = (rows: Record<string, number>, cli = 100, napi = 50): Side => ({
  rows,
  sizes: { "`vize` CLI (stripped)": cli, "`vize_vitrine` N-API library": napi },
});

void test("foldRuns keeps one count per row and rejects disagreeing executions", () => {
  const run = { a: { instructions: 5 }, b: { instructions: 7 } };
  assert.deepEqual(foldRuns({ runs: [run, run, run] }), { a: 5, b: 7 });
  assert.throws(
    () => foldRuns({ runs: [run, { a: { instructions: 6 }, b: { instructions: 7 } }] }),
    /disagree/,
  );
});

void test("movedRows lists rows that differ or exist on one side only", () => {
  assert.deepEqual(movedRows(side({ a: 1, b: 2 }), side({ a: 1, b: 3, c: 4 })), ["b", "c"]);
});

void test("renderReport tables every moved row, the identical rest, A/A and sizes", () => {
  const base = side({ a: 1_000, b: 2_000, c: 3_000 }, 1_000, 500);
  const head = side({ a: 990, b: 2_000, c: 3_003 }, 996, 500);
  const markdown = renderReport({
    baseSha: "a".repeat(40),
    headSha: "b".repeat(40),
    base,
    head,
    baseTwin: base,
    headTwin: head,
  });
  assert.match(markdown, /\| `a` \| 1,000 \| 990 \| −10 \(−1\.000%\) \| 1,000 \| 990 \|/);
  assert.match(markdown, /\| `c` \| 3,000 \| 3,003 \| \+3 \(\+0\.100%\) \|/);
  assert.match(markdown, /\| other 1 rows \| \| \| 0 \(identical\) \| \| \|/);
  assert.match(markdown, /\*\*2 moved, 1 up, 1 identical\*\* of 3 rows\./);
  assert.match(
    markdown,
    /A\/A: base vs base twin 0 moved; head vs head twin 0 moved; twin pair repeats every Δ above\./,
  );
  assert.match(markdown, /\| `vize` CLI \(stripped\) \| 1,000 \| 996 \| −4 \(−0\.400%\) \|/);
  assert.match(markdown, /\| `vize_vitrine` N-API library \| 500 \| 500 \| 0 \|/);
});

void test("renderReport works without twins", () => {
  const markdown = renderReport({
    baseSha: "a".repeat(40),
    headSha: "a".repeat(40),
    base: side({ a: 1 }),
    head: side({ a: 1 }),
  });
  assert.match(markdown, /\*\*0 moved, 0 up, 1 identical\*\* of 1 rows\./);
  assert.doesNotMatch(markdown, /A\/A:/);
});
