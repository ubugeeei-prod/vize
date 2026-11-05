import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

const raw = fs.readFileSync(
  new URL("../../_fixtures/tooling/sfc-style-raw-module-successors-20261105.json", import.meta.url),
);
assert.equal(
  createHash("sha256").update(raw).digest("hex"),
  "59289b52fbc5df2afe9df04b62d75e20241436e32078780260eac7e3f5bf50db",
);
export const sfcModuleSuccessors = JSON.parse(raw.toString()) as {
  source: string;
  run: number;
  artifact: number;
  qualificationCredit: boolean;
  previous: Array<Record<string, unknown> & { output: Record<string, unknown> }>;
  current: Array<Record<string, unknown> & { output: Record<string, unknown> }>;
};
const metrics = new Set(["arenaBytes", "expressionReparses", "counters", "phases"]);
/** Check actual whole packets before replaying the unchanged historical admissions. */
export function assertCurrentSfcModulePackets(rows: readonly Record<string, unknown>[]) {
  assert.deepEqual(
    rows.map((row) => Object.fromEntries(Object.entries(row).filter(([key]) => !metrics.has(key)))),
    sfcModuleSuccessors.current,
  );
  return structuredClone(sfcModuleSuccessors.previous);
}
