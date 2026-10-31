/** Live-base admission after #11172; the original historical transition remains immutable. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { contract } from "./jsx-child-backend-profile-contract.ts";
type Row = {
  source: string;
  ssr: boolean;
  iterationsPerSample: number;
  output: unknown;
  samplesNs: number[];
};
export function verifyCurrentBackendOutputs(before: Row[], after: Row[]): void {
  assert.equal(before.length, 6);
  assert.equal(after.length, 6);
  for (const [index, pinned] of contract.rows.entries()) {
    for (const row of [before[index]!, after[index]!]) {
      for (const key of ["source", "ssr", "iterationsPerSample"] as const)
        assert.deepEqual(row[key], pinned[key]);
      assert.equal(row.samplesNs.length, 9);
      assert.ok(row.samplesNs.every((n) => Number.isFinite(n) && n > 0));
      assert.deepEqual(row.output, pinned.after, `entire already-delivered backend output${index}`);
    }
    assert.deepEqual(before[index]!.output, after[index]!.output);
  }
}
export function run(directory: string): void {
  assert.ok(path.isAbsolute(directory));
  const before = JSON.parse(
    fs.readFileSync(path.join(directory, "base-profile.json"), "utf8"),
  ) as Row[];
  const after = JSON.parse(
    fs.readFileSync(path.join(directory, "head-profile.json"), "utf8"),
  ) as Row[];
  verifyCurrentBackendOutputs(before, after);
  fs.writeFileSync(
    path.join(directory, "comparison.json"),
    JSON.stringify(
      {
        scope:
          "all six current Vapor/SSR outputs equal the independently frozen #11172 successor; no default backend output transition in this VDOM-only change",
        historicalTransitionRequalified: false,
        runtimeQualified: false,
        performanceQualified: false,
        changedIndices: [],
        rows: after.map((row, index) => ({
          source: row.source,
          ssr: row.ssr,
          baseSamplesNs: before[index]!.samplesNs,
          headSamplesNs: row.samplesNs,
        })),
      },
      null,
      2,
    ) + "\n",
  );
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  run(process.argv[2]!);
