import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
import { admissionScalePlan, type ScaleAxis } from "./type-admission-scale-plan.ts";
import { captureAdmissionScale } from "./type-admission-scale-runner.ts";

const { values } = parseArgs({
  options: {
    provider: { type: "string" },
    out: { type: "string" },
    axis: { type: "string", default: "smoke" },
    sfcs: { type: "string", default: "2500" },
    tool: { type: "string", default: "all" },
  },
});
assert.ok(values.provider && values.out);
assert.ok(["all", "cli", "mapper", "vue-tsc"].includes(values.tool!));
const plan = admissionScalePlan(values.axis as ScaleAxis, Number(values.sfcs));
const tools =
  values.tool === "all"
    ? (["cli", "mapper", "vue-tsc"] as const)
    : [values.tool as "cli" | "mapper" | "vue-tsc"];
const out = path.resolve(values.out);
fs.mkdirSync(out, { recursive: true });
fs.writeFileSync(
  path.join(out, "plan.json"),
  JSON.stringify(
    {
      source: process.env.GITHUB_SHA,
      plan,
      tools,
      scope:
        "Original public fixtures; published provider baseline, no private corpus or source-binary qualification",
    },
    null,
    2,
  ),
);
const results: unknown[] = [];
let passed = true;
// A failed or unsupported point keeps its raw error packet and does not erase
// the rest of the matrix. Legitimate TS depth-limit errors remain diagnostics.
for (const row of plan)
  for (const tool of tools)
    for (const stage of ["valid", "invalid", "poisoned"] as const) {
      const directory = path.join(out, row.name, tool, stage);
      try {
        const receipt = captureAdmissionScale({
          provider: path.resolve(values.provider),
          out: directory,
          tool,
          scale: { ...row.options, stage },
        });
        const result = {
          name: row.name,
          tool,
          stage,
          passed: receipt.accuracy.passed,
          elapsedMs: receipt.elapsedMs,
          maxProcessRssBytes: receipt.maxProcessRssBytes,
          receipt: path.join(directory, "receipt.json"),
        };
        results.push(result);
        passed &&= result.passed;
        console.log(JSON.stringify(result));
      } catch (error) {
        passed = false;
        const failure = {
          name: row.name,
          tool,
          stage,
          passed: false,
          error: error instanceof Error ? error.stack : String(error),
        };
        fs.mkdirSync(directory, { recursive: true });
        fs.writeFileSync(path.join(directory, "failure.json"), JSON.stringify(failure, null, 2));
        results.push(failure);
        console.log(JSON.stringify(failure));
      }
      fs.writeFileSync(
        path.join(out, "results.json"),
        JSON.stringify({ passed, expectedRuns: plan.length * tools.length * 3, results }, null, 2),
      );
    }
assert.equal(results.length, plan.length * tools.length * 3);
if (!passed) process.exitCode = 1;
