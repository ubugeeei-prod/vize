/** Finite full-output transitions for real JSX child work; no timing-equivalence claim. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
type Output = null | boolean | number | string | Output[] | { [key: string]: Output };
type Row = {
  source: string;
  ssr: boolean;
  iterationsPerSample: number;
  output: Output;
  samplesNs: number[];
};
type Pinned = {
  index: number;
  source: string;
  ssr: boolean;
  iterationsPerSample: number;
  before: Output;
  after: Output;
};
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const bytes = fs.readFileSync(
  new URL(
    "../../_fixtures/differential/jsx/child-values-6890-backend-profile-contracts.json",
    import.meta.url,
  ),
);
assert.equal(hash(bytes), "5e9cc1e55a43bb2edc5e8b08880e96633401332d9b0d3eee9e845857b60cb6a1");
export const contract = JSON.parse(bytes.toString()) as {
  schema: number;
  changedIndices: number[];
  qualification: unknown;
  rows: Pinned[];
};
assert.equal(contract.schema, 1);
assert.deepEqual(contract.changedIndices, [0, 2, 4]);
assert.deepEqual(contract.qualification, { runtime: false, performance: false });
assert.deepEqual(
  contract.rows.map((row) => row.index),
  [0, 1, 2, 3, 4, 5],
);
export function verifyBackendOutputs(before: Row[], after: Row[]): number[] {
  assert.equal(before.length, 6);
  assert.equal(after.length, 6);
  const changed: number[] = [];
  for (const [index, pinned] of contract.rows.entries()) {
    for (const role of [before[index], after[index]]) {
      for (const key of ["source", "ssr", "iterationsPerSample"] as const)
        assert.deepEqual(role[key], pinned[key]);
      assert.equal(role.samplesNs.length, 9);
      assert(role.samplesNs.every((value) => Number.isFinite(value) && value > 0));
    }
    assert.deepEqual(before[index].output, pinned.before, `whole original backend output${index}`);
    assert.deepEqual(
      after[index].output,
      pinned.after,
      `whole exact successor backend output${index}`,
    );
    if (JSON.stringify(pinned.before) !== JSON.stringify(pinned.after)) {
      assert.equal(pinned.ssr, false);
      changed.push(index);
    } else assert.deepEqual(before[index].output, after[index].output);
  }
  assert.deepEqual(changed, [0, 2, 4]);
  return changed;
}
export function run(directory: string): void {
  assert(path.isAbsolute(directory));
  const beforeBytes = fs.readFileSync(path.join(directory, "base-profile.json"));
  const afterBytes = fs.readFileSync(path.join(directory, "head-profile.json"));
  const before = JSON.parse(beforeBytes.toString()) as Row[],
    after = JSON.parse(afterBytes.toString()) as Row[];
  const changed = verifyBackendOutputs(before, after);
  const median = (values: number[]) =>
    values.toSorted((a, b) => a - b)[Math.floor(values.length / 2)];
  const rows = after.map((row, index) => ({
    source: row.source,
    ssr: row.ssr,
    baseMedianNs: median(before[index].samplesNs),
    headMedianNs: median(row.samplesNs),
    ratio: median(row.samplesNs) / median(before[index].samplesNs),
    intentionalChildValueChange: changed.includes(index),
    equivalentWorkTiming: !changed.includes(index),
  }));
  fs.writeFileSync(
    path.join(directory, "comparison.json"),
    JSON.stringify(
      {
        scope:
          "default backend entrypoints; exact three JSX child output transitions; full compile including parsing/lowering; changed-work timings descriptive only",
        performanceQualified: false,
        runtimeQualified: false,
        source: process.env.QUALIFICATION_HEAD_SHA ?? null,
        baseProfileSha256: hash(beforeBytes),
        headProfileSha256: hash(afterBytes),
        rows,
      },
      null,
      2,
    ) + "\n",
  );
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  run(process.argv[2]);
