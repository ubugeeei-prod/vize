/** Repeated-source timing and the exact pinned retry-class contract. */
import assert from "node:assert/strict";
import path from "node:path";
import {
  integer,
  isMain,
  readExact,
  record,
  render,
  writeExact,
  type RecordValue,
} from "./profile-evidence-json.ts";
import { validate } from "./recovery_cost_contract.ts";

function signature(row: RecordValue): unknown[] {
  return ["sourceHash", "sourceBytes", "checksum", "iterations", "scenario"].map((key) => row[key]);
}
function median(rows: bigint[]): bigint {
  assert(rows.length > 0 && rows.length % 2 === 1);
  return [...rows].sort((left, right) => (left < right ? -1 : left > right ? 1 : 0))[
    Math.floor(rows.length / 2)
  ]!;
}
export function main(): void {
  const root = path.resolve(process.argv[2] ?? "evidence");
  const normal: Record<string, unknown> = {};
  for (const scenario of ["valid", "incomplete-member"]) {
    const cases: Record<string, unknown> = {};
    for (const rows of [32, 256, 2048]) {
      const data: Record<string, RecordValue[]> = {};
      for (const role of ["base", "head"])
        data[role] = [1, 2, 3].map((round) =>
          record(
            readExact(
              path.join(
                root,
                "normal-" + scenario + "-" + role + "-" + rows + "-" + round + ".json",
              ),
            ),
          ),
        );
      const expected = signature(data.base![0]!);
      for (const samples of Object.values(data))
        for (const row of samples) assert.deepEqual(signature(row), expected);
      cases[rows] = Object.fromEntries(
        Object.entries(data).map(([role, samples]) => {
          const samplesNs = samples.map((row) => integer(row.elapsedNs));
          return [role, { samplesNs, medianNs: median(samplesNs) }];
        }),
      );
    }
    normal[scenario] = cases;
  }
  const profiles = Object.fromEntries(
    ["base", "head"].map((role) => [
      role,
      record(readExact(path.join(root, "profile-" + role + ".json"))),
    ]),
  );
  assert.deepEqual(profiles.base!.sourceHash, profiles.head!.sourceHash);
  const summary = {
    scope: "repeated source parse+analysis, not cold whole-project time",
    normal,
    recoveryComparison: validate(profiles),
    allocation: Object.fromEntries(
      Object.entries(profiles).map(([role, row]) => [role, row.allocation]),
    ),
  };
  writeExact(path.join(root, "summary.json"), summary);
  console.log(
    "comparisonReceipt=" +
      render({ identity: readExact(path.join(root, "identity.json")), comparison: summary }),
  );
}
if (isMain(import.meta.url)) main();
