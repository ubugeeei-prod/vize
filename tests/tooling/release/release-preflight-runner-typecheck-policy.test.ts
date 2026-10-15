import assert from "node:assert/strict";
import { test } from "node:test";

import { runVerifyOnlyPreflight as actualPreflight } from "../support/release-preflight-verify-only.ts";

function mutateLintBudget(budget: Record<string, unknown>) {
  return (entries: Record<string, string>) => {
    const report = JSON.parse(entries["lint-divergence-summary.json"]);
    Object.assign(report.budget, budget);
    entries["lint-divergence-summary.json"] = JSON.stringify(report);
  };
}

test("release preflight ignores recorded matrix parity gaps", () => {
  for (const mutate of [
    (entries: Record<string, string>) => {
      for (const name of Object.keys(entries))
        if (name.endsWith("-typecheck-divergence.json")) delete entries[name];
    },
    mutateLintBudget({
      status: "failure",
      passed: false,
      passedCount: 0,
      failedCount: 1,
      breachedCount: 1,
      failedProjects: ["fixture"],
    }),
    (entries: Record<string, string>) => {
      const name = Object.keys(entries).find((entry) =>
        entry.endsWith("-typecheck-divergence.json"),
      );
      assert.ok(name);
      const report = JSON.parse(entries[name]);
      report.divergence.summary.falseNegativeCount = 1;
      entries[name] = JSON.stringify(report);
    },
  ]) {
    const result = actualPreflight(mutate);
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /Release preflight passed/);
  }
});

test("release preflight ignores a matrix lint budget that hides measured projects", () => {
  for (const budget of [
    { status: "failure", passed: false, failedCount: 1 },
    { status: "success", passed: true, passedCount: 0, failedCount: 1, breachedCount: 1 },
    { passedCount: 0, failedCount: 1, breachedCount: 1, failedProjects: ["fixture"] },
    { projectCount: 2, passedCount: 2 },
  ]) {
    const result = actualPreflight(mutateLintBudget(budget));
    assert.equal(result.status, 0, JSON.stringify(budget));
    assert.match(result.stdout, /Release preflight passed/);
  }
});
