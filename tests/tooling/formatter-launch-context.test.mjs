import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import {
  AFFECTED,
  fixedCounts,
  inspectDump,
  prepareProbes,
  replayPlan,
  summarizeContexts,
} from "../../tools/benchmarks/scripts/formatter-launch-context.mjs";

test("copied bytes use one regular fixed executable and old roles remain symlinks", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "formatter-launch-test-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const file = path.join(root, "preserved");
  const bytes = Buffer.from("one authenticated binary fixture");
  fs.writeFileSync(file, bytes);
  const hash = createHash("sha256").update(bytes).digest("hex");
  const plan = replayPlan(root);
  prepareProbes(plan, file, hash);
  for (const row of plan) {
    assert.deepEqual(fs.readFileSync(row.executable), bytes);
    assert.equal(fs.lstatSync(row.executable).isSymbolicLink(), row.method === "old");
  }
  assert.throws(() => prepareProbes(plan, file, hash), /existing launch context/);
});

test("launch diagnosis reuses one fixed regular execution path and mirrors the old five-byte role drift", () => {
  const plan = replayPlan("/home/runner/_work/_temp");
  assert.equal(plan.length, 12);
  const fixed = plan.filter((run) => run.method === "fixed");
  assert.equal(new Set(fixed.map((run) => run.executable)).size, 1);
  assert.deepEqual([...new Set(plan.map((run) => run.repetition))], [1, 2, 3]);
  const old = plan.filter((run) => run.method === "old");
  assert.equal(old.find((run) => run.role === "old-primary").executable.length, 97);
  assert.equal(old.find((run) => run.role === "old-twin").executable.length, 102);
  for (const role of ["old-primary", "old-twin", "fixed-primary", "fixed-twin"])
    assert.equal(plan.filter((run) => run.role === role).length, 3);
});

test("observed guest argv0 is verified rather than inferred from the requested recipe", () => {
  const dump = `# callgrind format\nversion: 1\ncmd:  /fixed/probe-00\ndesc: Trigger: Client Request: ${AFFECTED}\npositions: line\nevents: Ir\nsummary: 0\nfn=probe\n1 13\ntotals: 13\n`;
  assert.equal(inspectDump(dump, "/fixed/probe-00").argv0, "/fixed/probe-00");
  assert.throws(() => inspectDump(dump, "/old/probe-00"), /guest command/);
  assert.throws(
    () => inspectDump(dump.replace("cmd:", "unknown:"), "/fixed/probe-00"),
    /guest command/,
  );
});

test("the diagnostic keeps old instability and rejects instability in the six fixed-path observations", () => {
  const runs = replayPlan("/tmp/job").map((run) => ({
    ...run,
    rows: {
      [AFFECTED]: { instructions: run.method === "old" && run.role === "old-twin" ? 114 : 100 },
    },
  }));
  assert.deepEqual(fixedCounts(runs), [100]);
  assert.equal(summarizeContexts(runs).oldDriftReproduced, true);
  const stableBoth = runs.map((run) => ({ ...run, rows: { [AFFECTED]: { instructions: 100 } } }));
  assert.equal(summarizeContexts(stableBoth).oldDriftReproduced, false);
  assert.equal(summarizeContexts(stableBoth).fixedPathStable, true);
  runs.find((run) => run.role === "fixed-twin").rows[AFFECTED].instructions = 101;
  assert.deepEqual(fixedCounts(runs), [100, 101]);
  assert.throws(() => fixedCounts(runs.slice(1, 4)), /three fixed primary/);
});
