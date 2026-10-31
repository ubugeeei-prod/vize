/** Exact generated-output transitions; raw timing samples are never rewritten. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type Json = null | string | number | boolean | Json[] | { [key: string]: Json };
type Row = { source: string; ssr: boolean; map: boolean; output: Json };
type Contract = {
  schema: number;
  originalRows: number;
  childIndices: number[];
  existingStaticIndices: number[];
  qualification: { runtime: boolean; performance: boolean };
  rows: { index: number; source: string; ssr: boolean; map: boolean; before: Json; after: Json }[];
  existingStaticRows: {
    index: number;
    source: string;
    ssr: boolean;
    map: boolean;
    before: Json;
    after: Json;
  }[];
};
const hash = (bytes: Uint8Array | string) => createHash("sha256").update(bytes).digest("hex");
const bytes = fs.readFileSync(
  new URL(
    "../../_fixtures/differential/jsx/child-values-6890-profile-contracts.json",
    import.meta.url,
  ),
);
assert.equal(hash(bytes), "2ec2184e237da0d2c7fb7e09a83fb3c23cb06fce2bdfde057daefb61c3bc4553");
export const contract = JSON.parse(bytes.toString()) as Contract;
assert.equal(contract.schema, 1);
assert.equal(contract.originalRows, 16);
assert.deepEqual(contract.childIndices, [0, 1, 8, 9]);
assert.deepEqual(contract.existingStaticIndices, [12, 13]);
assert.deepEqual(contract.qualification, { runtime: false, performance: false });
assert.deepEqual(
  contract.rows.map((row) => row.index),
  contract.childIndices,
);
assert.deepEqual(
  contract.existingStaticRows.map((row) => row.index),
  [12, 13],
);

export function verifyProfileOutputs(before: Row[], after: Row[]): number[] {
  assert.equal(before.length, 16);
  assert.equal(after.length, 16);
  const intentional: number[] = [];
  for (const [index, current] of after.entries()) {
    const previous = before[index];
    for (const key of ["source", "ssr", "map"] as const) assert.equal(previous[key], current[key]);
    const entry = contract.rows.find((row) => row.index === index);
    if (entry) {
      for (const key of ["source", "ssr", "map"] as const) assert.equal(current[key], entry[key]);
      assert.deepEqual(
        previous.output,
        entry.before,
        `complete original child contract at${index}`,
      );
      assert.deepEqual(current.output, entry.after, `complete successor child contract at${index}`);
      assert.equal(current.ssr, false);
      intentional.push(index);
    } else {
      const existingStatic = current.source.startsWith("const Child={};") && !current.ssr;
      assert.equal(existingStatic, contract.existingStaticIndices.includes(index));
      if (existingStatic) {
        const pinned = contract.existingStaticRows.find((row) => row.index === index);
        assert(pinned, `missing exact static output at${index}`);
        for (const key of ["source", "ssr", "map"] as const)
          assert.equal(current[key], pinned[key]);
        assert.deepEqual(previous.output, pinned.before);
        assert.deepEqual(current.output, pinned.after);
      } else assert.deepEqual(previous.output, current.output);
    }
  }
  assert.deepEqual(intentional, [0, 1, 8, 9]);
  return intentional;
}

export function run(mode: string, directory: string): void {
  assert.ok(path.isAbsolute(directory));
  const beforePath = path.join(directory, "stability-base-profile.json");
  const afterPath = path.join(directory, "head-profile.json");
  const receiptPath = path.join(directory, "child-profile-contracts.json");
  const beforeBytes = fs.readFileSync(beforePath),
    afterBytes = fs.readFileSync(afterPath);
  if (mode === "verify") {
    const intentional = verifyProfileOutputs(
      JSON.parse(beforeBytes.toString()) as Row[],
      JSON.parse(afterBytes.toString()) as Row[],
    );
    fs.writeFileSync(
      receiptPath,
      JSON.stringify(
        {
          schema: 1,
          originalRows: 16,
          intentionalChildIndices: intentional,
          existingStaticIndices: [12, 13],
          beforeSha256: hash(beforeBytes),
          afterSha256: hash(afterBytes),
          source: process.env.QUALIFICATION_HEAD_SHA ?? null,
          rawProfilesUnmodified: true,
          runtimeQualified: false,
          performanceQualified: false,
        },
        null,
        2,
      ) + "\n",
    );
  } else {
    assert.equal(mode, "report");
    const receipt = JSON.parse(fs.readFileSync(receiptPath, "utf8")) as {
      beforeSha256: string;
      afterSha256: string;
      intentionalChildIndices: number[];
    };
    assert.equal(hash(beforeBytes), receipt.beforeSha256);
    assert.equal(hash(afterBytes), receipt.afterSha256);
    const file = path.join(directory, "stability-comparison.json");
    const report = JSON.parse(fs.readFileSync(file, "utf8")) as {
      scope: string;
      rows: Record<string, Json>[];
      performanceQualified?: boolean;
    };
    assert.equal(report.rows.length, 16);
    report.scope =
      "literal-guarded664 versus head; four exact child-code transitions and the original static-value pair are explicit; raw timings are descriptive, without equivalent-work speedup credit";
    report.performanceQualified = false;
    report.rows.forEach((row, index) => {
      row.intentionalChildValueChange = receipt.intentionalChildIndices.includes(index);
    });
    fs.writeFileSync(file, JSON.stringify(report, null, 2) + "\n");
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [mode, directory] = process.argv.slice(2);
  assert.ok(mode && directory);
  run(mode, directory);
}
