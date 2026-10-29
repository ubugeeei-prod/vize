import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";

interface Step {
  name?: string;
  run?: string;
}
interface Job {
  steps: Step[];
}
interface Workflow {
  jobs: Record<string, Job>;
}
const source = readFileSync(
  new URL("../../.github/workflows/native-ops-incident.yml", import.meta.url),
  "utf8",
);
const workflow = parse(source) as Workflow;
// Exact parsed forensic job from reviewed fb244 / workflow blob edf7edfd.
// Adding the hosted-only debugger must not mutate the original710 recipe.
const forensicRecipeSHA256 = "4951fd590d716446a8bcb2e2fceb1033fa9a52cc171fdbe0b3aabb8caf0521e3";

function assertBoundary(value: Workflow): void {
  const control = value.jobs["observer-control"];
  const forensic = value.jobs["forensic-only"];
  assert.ok(control && forensic, "Both diagnostic stages must remain explicit");
  const packageSteps = Object.entries(value.jobs).flatMap(([job, entry]) =>
    entry.steps.flatMap((step) => (/\bapt-get\b/u.test(step.run ?? "") ? [{ job, step }] : [])),
  );
  assert.equal(packageSteps.length, 1, "Exactly one synthetic tool-install step");
  assert.equal(packageSteps[0]?.job, "observer-control", "No target runtime package mutation");
  assert.equal(
    packageSteps[0]?.step.run,
    "sudo apt-get update\nsudo apt-get install --no-install-recommends --no-upgrade gdb\n",
  );
  assert.equal(
    createHash("sha256").update(JSON.stringify(forensic)).digest("hex"),
    forensicRecipeSHA256,
    "The complete reviewed original710 job must remain unchanged",
  );
}

void test("debugger setup belongs only to hosted synthetic controls", () => {
  assertBoundary(workflow);
});
void test("package mutation on the forensic target is rejected", () => {
  const changed = structuredClone(workflow);
  assert.ok(changed.jobs["forensic-only"]);
  changed.jobs["forensic-only"].steps.push({ run: "sudo apt-get install gdb\n" });
  assert.throws(() => assertBoundary(changed));
});
void test("changing the original target command is rejected", () => {
  const changed = structuredClone(workflow);
  const step = changed.jobs["forensic-only"]?.steps.find(
    (entry) => entry.name === "Run the original source command once",
  );
  assert.ok(step?.run);
  step.run = "RUST_MIN_STACK=16777216 " + step.run;
  assert.throws(() => assertBoundary(changed));
});
void test("dropping synthetic debugger setup is rejected", () => {
  const changed = structuredClone(workflow);
  const control = changed.jobs["observer-control"];
  assert.ok(control);
  control.steps = control.steps.filter((step) => !/\bapt-get\b/u.test(step.run ?? ""));
  assert.throws(() => assertBoundary(changed));
});
