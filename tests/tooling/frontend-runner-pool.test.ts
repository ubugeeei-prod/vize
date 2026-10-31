import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";

type Step = { name?: string; if?: string; [key: string]: unknown };
type Job = { "runs-on"?: string; if?: string; steps?: Step[]; [key: string]: unknown };
type Workflow = { jobs: Record<string, Job>; [key: string]: unknown };
const oldRunner = "blacksmith-32vcpu-ubuntu-2404";
const hosted = "ubuntu-24.04";
const fastEvents = ["pull_request", "merge_group", "push"];
const selectors = [
  {
    name: "check",
    jobs: ["iteration-plan", "fmt-rust", "check-js", "security-audit", "node-engine-compat"],
    original: "0577d82da5d8cd83b109422085d84902edb51c0a0e45407f09301f54c9a1b451",
  },
  {
    name: "pr-source-checks",
    jobs: ["pr-source-plan", "ci-selection-tests"],
    original: "660eee19156f5ec124d70852c5777ec2a95025f14aa95cd0b6b07ce33d8a1007",
  },
] as const;
const workflows = selectors.map((specification) => {
  const before = fs.readFileSync(
    new URL(
      "./fixtures/frontend-runner-pool/" + specification.name + ".before.yml.txt",
      import.meta.url,
    ),
    "utf8",
  );
  assert.equal(createHash("sha256").update(before).digest("hex"), specification.original);
  const after = fs.readFileSync(
    new URL("../../.github/workflows/" + specification.name + ".yml", import.meta.url),
    "utf8",
  );
  return { specification, before: parse(before) as Workflow, after: parse(after) as Workflow };
});
function expression(value: string) {
  return value
    .trim()
    .replace(/^\$\{\{\s*/, "")
    .replace(/\s*\}\}$/, "")
    .replace(
      /\.([A-Za-z_][A-Za-z0-9_]*-[A-Za-z0-9_-]+)/g,
      (_match, key: string) => '["' + key + '"]',
    );
}
function evaluate(value: string, context: Record<string, unknown>): unknown {
  return runInNewContext(expression(value), context, { timeout: 100 });
}
function context(event: string, dependencies: string, cancelled: boolean) {
  return {
    github: { event_name: event, ref: "refs/heads/main" },
    inputs: { "release-qualification": false },
    needs: {
      "iteration-plan": { outputs: { dependencies, engines: dependencies } },
      "pr-source-plan": { outputs: { tooling: dependencies }, result: "success" },
    },
    always: () => true,
    cancelled: () => cancelled,
  };
}

const providerStep = {
  name: "Provision the locked Full Rust native runtime",
  run: "node tools/support/compat/github/full-rust-runtime.ts",
};
const providerArtifact = {
  name: "Retain source coverage native runtime provider",
  if: "${{ always() }}",
  uses: "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a",
  with: {
    name: "source-coverage-native-runtime-${{ github.sha }}-${{ github.run_id }}-${{ github.run_attempt }}",
    path: "target/nextest/full/native-runtime-provider.json",
    "if-no-files-found": "warn",
    "retention-days": 14,
  },
};

function restoreCoverageBeforeRuntimeProvisioning(job: Job) {
  const steps = job.steps;
  assert(steps);
  for (const [expected, before, after] of [
    [providerStep, "Install cargo-llvm-cov", "Source coverage report"],
    [
      providerArtifact,
      "Retain raw source coverage native syntax packets",
      "Verify source coverage summary",
    ],
  ] as const) {
    const matches: number[] = steps.flatMap((step, index) =>
      step.name === expected.name ? [index] : [],
    );
    assert.equal(matches.length, 1, "exactly one reviewed provider step is required");
    const index = matches[0];
    assert.deepEqual(steps[index], expected);
    assert.equal(steps[index - 1]?.name, before);
    assert.equal(steps[index + 1]?.name, after);
    steps.splice(index, 1);
  }
}

test("the seven runner selectors and exact runtime steps preserve every original job", () => {
  let changed = 0,
    protectedJobs = 0;
  for (const { specification, before, after } of workflows) {
    const restored = structuredClone(after);
    if (specification.name === "check")
      restoreCoverageBeforeRuntimeProvisioning(restored.jobs["source-coverage"]);
    for (const id of specification.jobs) {
      const original = before.jobs[id],
        updated = after.jobs[id],
        target = restored.jobs[id];
      assert(original && updated && target, id);
      assert.equal(original["runs-on"], oldRunner);
      assert.equal(typeof updated["runs-on"], "string");
      if (id !== "check-js") assert.equal(updated["runs-on"], hosted);
      target["runs-on"] = oldRunner;
      changed += 1;
    }
    for (const [id, job] of Object.entries(before.jobs)) {
      if (!specification.jobs.some((selected) => selected === id)) {
        assert.deepEqual(restored.jobs[id], job, specification.name + "/" + id);
        protectedJobs += 1;
      }
    }
    const originalJobs = Object.fromEntries(
      Object.keys(before.jobs).map((id) => [id, restored.jobs[id]]),
    );
    assert.deepEqual(
      originalJobs,
      before.jobs,
      "all original job commands, toolchains, needs, predicates, outputs and budgets",
    );
  }
  assert.equal(changed, 7);
  assert.equal(protectedJobs, 25);
});

test("source coverage refuses unknown provider mutations while retaining the archived workflow", () => {
  const current = workflows.find((entry) => entry.specification.name === "check")?.after.jobs[
    "source-coverage"
  ];
  assert(current);
  for (const mutate of [
    (job: Job) => job.steps?.push(structuredClone(providerStep)),
    (job: Job) => job.steps?.push(structuredClone(providerArtifact)),
    (job: Job) => {
      const step = job.steps?.find((step) => step.name === providerStep.name);
      assert(step);
      step.run = "node unreviewed-provider.ts";
    },
    (job: Job) => {
      const step = job.steps?.find((step) => step.name === providerArtifact.name);
      assert(step);
      step.if = "${{ success() }}";
    },
    (job: Job) => {
      const index = job.steps?.findIndex((step) => step.name === providerStep.name);
      assert(index != null && index >= 0 && job.steps);
      job.steps.splice(index, 1);
    },
  ]) {
    const changed = structuredClone(current);
    mutate(changed);
    assert.throws(() => restoreCoverageBeforeRuntimeProvisioning(changed), assert.AssertionError);
  }
});

test("all existing frontend job and step truth tables stay identical", () => {
  let vectors = 0;
  for (const { specification, before, after } of workflows) {
    for (const id of specification.jobs) {
      const original = before.jobs[id],
        updated = after.jobs[id];
      assert(original && updated, id);
      const predicates = [{ before: original.if ?? "true", after: updated.if ?? "true" }];
      for (const [index, step] of (original.steps ?? []).entries()) {
        const counterpart = updated.steps?.[index];
        assert(counterpart, id + "/step" + index);
        predicates.push({ before: step.if ?? "true", after: counterpart.if ?? "true" });
      }
      for (const event of [...fastEvents, "schedule", "workflow_dispatch"]) {
        for (const dependencies of ["true", "false"]) {
          for (const cancelled of [false, true]) {
            const input = context(event, dependencies, cancelled);
            for (const predicate of predicates) {
              const expected = evaluate(predicate.before, input);
              assert.equal(typeof expected, "boolean");
              assert.equal(evaluate(predicate.after, input), expected);
              vectors += 1;
            }
          }
        }
      }
    }
  }
  assert.equal(vectors, 940);
});

test("full scheduled and manual native producers keep their original runner", () => {
  const check = workflows.find((entry) => entry.specification.name === "check");
  const selector = check?.after.jobs["check-js"]?.["runs-on"];
  assert(selector);
  for (const event of [...fastEvents, "schedule", "workflow_dispatch"]) {
    const expected = fastEvents.includes(event) ? hosted : oldRunner;
    assert.equal(evaluate(selector, context(event, "true", false)), expected, event);
  }
});
