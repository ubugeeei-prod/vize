import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

type JsonRecord = Record<string, unknown>;
type Row = { session: string; exitCode: number | null; comparable: boolean;
  authorityErrors: string[]; completionErrors: string[] };
function record(value: unknown): JsonRecord {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  return Object.fromEntries(Object.entries(value));
}
function read(file: string): JsonRecord {
  const value: unknown = JSON.parse(fs.readFileSync(file, "utf8"));
  return record(value);
}
function length(value: unknown): number {
  assert(Array.isArray(value));
  return value.length;
}
function array(value: unknown): unknown[] {
  assert(Array.isArray(value));
  return value;
}
function message(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
const output = process.env["EVIDENCE"];
const source = process.env["SOURCE_SHA"];
const role = process.env["ROLE"];
assert(typeof output === "string" && typeof source === "string");
assert(role === "base" || role === "candidate");
const rows: Row[] = [];
const rssLabels = ["coldOpen", ...Array.from({ length: 20 }, (_, index: number) => `A${index}`),
  ...Array.from({ length: 20 }, (_, index: number) => `B${index}`), "cancellation", "closed",
  ...Array.from({ length: 20 }, (_, index: number) => `file-lifecycle-${index}`)];
for (const session of ["incremental", "churn-1", "churn-2", "churn-3"]) {
  const row: Row = { session, exitCode: null, comparable: false,
    authorityErrors: [], completionErrors: [] };
  rows.push(row);
  try {
    const code = read(path.join(output, session, "exit.json"))["exitCode"];
    assert(typeof code === "number" && Number.isSafeInteger(code));
    row.exitCode = code;
    if (session.startsWith("churn")) {
      const file = path.join(output, session, "metrics/misskey-lsp-churn/metrics.json");
      if (!fs.existsSync(file)) { row.completionErrors.push("metrics absent; observation only"); continue; }
      const metrics = read(file);
      assert.equal(metrics["commit"], source);
      const budget = record(metrics["budget"]);
      assert.equal(budget["scale"], 1);
      assert.equal(budget["maxPeakRssMiB"], 136);
      assert.equal(budget["cyclesPerPhase"], 20);
      assert(metrics["status"] === "passed" || metrics["status"] === "failed");
      try {
        const timings = record(metrics["timingsMs"]);
        assert.equal(length(timings["cycle"]), 40);
        assert.equal(length(timings["fileLifecycle"]), 20);
        assert.equal(record(metrics["cycleStats"])["cycles"], 40);
        assert.deepEqual(array(metrics["rssSamples"]).map((sample: unknown) => record(sample)["label"]), rssLabels);
        row.comparable = true;
        if (role === "candidate") assert.equal(metrics["status"], "passed");
      } catch (error: unknown) { row.completionErrors.push(message(error)); }
    } else {
      row.comparable = code === 0;
    }
  } catch (error: unknown) {
    row.authorityErrors.push(message(error));
  }
}
fs.writeFileSync(path.join(output, "results.json"), JSON.stringify(rows, null, 2) + "\n");
assert(rows.every((row: Row) => row.authorityErrors.length === 0), "retained evidence must have valid authority");
if (role === "candidate") assert(rows.every((row: Row) => row.exitCode === 0 && row.comparable
  && row.completionErrors.length === 0), "every complete original candidate session must pass");
