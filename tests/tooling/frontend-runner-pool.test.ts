import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";
import { restoreReviewedCheckout } from "./support/frontend-checkout-contract.ts";
import {
  restoreCheckJsCasePacket,
  restoreHistoryToolchainSelection,
  restoreNativeRuntimeCasePacket,
  restoreNodeEngineCasePacket,
} from "./support/frontend-case-reporting.ts";
import { restorePlaygroundCaseReporting } from "./support/frontend-playground-reporting.ts";
import { helperAbiStep, restoreContentMapperHelperAbi } from "./support/frontend-helper-abi.ts";
import { restorePinnedWildAction } from "./support/github-workflows.ts";

type Step = { name?: string; uses?: string; if?: string; with?: Record<string, unknown> } & Record<
  string,
  unknown
>;
type Job = { "runs-on"?: string; if?: string; steps?: Step[] } & Record<string, unknown>;
type Workflow = { jobs: Record<string, Job> };
const maximumRunner = "blacksmith-32vcpu-ubuntu-2404";
const hosted = "ubuntu-24.04";
const fastEvents = ["pull_request", "merge_group", "push"];
const selectors = [
  {
    name: "check",
    file: ".github/workflows/check.yml",
    jobs: ["iteration-plan", "fmt-rust", "check-js", "security-audit", "node-engine-compat"],
    original: "0577d82da5d8cd83b109422085d84902edb51c0a0e45407f09301f54c9a1b451",
  },
  {
    name: "pr-source-checks",
    file: ".github/workflows/pr-source-checks.yml",
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
  const after = fs.readFileSync(new URL("../../" + specification.file, import.meta.url), "utf8");
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

function restoreMaximumRunner(job: Job, original: Job) {
  if (job["runs-on"] === original["runs-on"]) return false;
  assert.equal(original["runs-on"], hosted);
  assert.equal(job["runs-on"], maximumRunner);
  job["runs-on"] = original["runs-on"];
  return true;
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

function restoreEmptyPrToolingBuild(workflow: Workflow, original: Workflow) {
  const producer = "affected-tooling-cli";
  assert.deepEqual(workflow.jobs[producer], {
    ...original.jobs[producer],
    needs: "pr-source-plan",
    if: "${{ github.event_name != 'pull_request' || needs.pr-source-plan.outputs.affected-tooling != 'false' }}",
  });
  workflow.jobs[producer] = structuredClone(original.jobs[producer]);
  const report = workflow.jobs["affected-tooling-report"];
  const before = original.jobs["affected-tooling-report"];
  restoreMaximumRunner(report, before);
  assert(report.steps && report.steps.length === 1);
  const step = report.steps[0];
  const previous = before.steps?.[0];
  assert(step && previous && typeof step.run === "string");
  assert.deepEqual(step.env, {
    EVENT_NAME: "${{ github.event_name }}",
    ...(previous.env as Record<string, string>),
  });
  assert.equal(
    createHash("sha256").update(step.run).digest("hex"),
    "411d4d38167ab8f1a4a506a15cf2556ca6d5a0411659d885fe63d2f7bee7970e",
    "only the separately executed four-state report is reviewed",
  );
  step.env = structuredClone(previous.env);
  step.run = previous.run;
  assert.deepEqual(report, before);
}

// #11652 posts the PR test report after the unchanged gate; invert exactly that.
const testReportSteps = [
  {
    name: "Download test case reports",
    id: "cases",
    if: "${{ !cancelled() }}",
    "continue-on-error": true,
    uses: "actions/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c",
    with: { pattern: "test-cases-*", path: "${{ runner.temp }}/test-cases" },
  },
  {
    name: "Report test results",
    if: "${{ !cancelled() }}",
    "continue-on-error": true,
    env: {
      GITHUB_TOKEN: "${{ github.token }}",
      TEST_REPORT_GATE: "${{ steps.gate.outcome }}",
      TEST_REPORT_CASES: "${{ steps.cases.outcome }}",
      TEST_REPORT_CASES_DIR: "${{ runner.temp }}/test-cases",
      TEST_REPORT_HEAD_SHA:
        "${{ github.event.pull_request.head.sha || github.event.merge_group.head_sha }}",
      TEST_REPORT_PR_NUMBER:
        "${{ github.event.pull_request.head.repo.full_name == github.repository && github.event.pull_request.number || '' }}",
    },
    run: "node tools/support/compat/github/comment-test-report.ts",
  },
];

function restoreTestReportComment(workflow: Workflow, original: Workflow) {
  const before = original.jobs["test-report"];
  restoreMaximumRunner(workflow.jobs["test-report"], before);
  const steps = before.steps ?? [];
  const gate = steps.at(-1);
  assert.equal(gate?.name, "Require every needed job to succeed");
  assert.deepEqual(workflow.jobs["test-report"], {
    ...before,
    permissions: { ...(before.permissions as Record<string, string>), "pull-requests": "write" },
    steps: [...steps.slice(0, -1), { ...gate, id: "gate" }, ...testReportSteps],
  });
  workflow.jobs["test-report"] = structuredClone(before);
}

function restoreToolingCasePacketJob(workflow: Workflow, original: Workflow) {
  const before = original.jobs["affected-tooling"];
  const steps = before.steps ?? [];
  const worker = steps.at(-1);
  assert.equal(worker?.uses, "./.github/actions/test-affected-tooling");
  assert.deepEqual(workflow.jobs["affected-tooling"], {
    ...before,
    steps: [
      ...steps.slice(0, -1),
      {
        ...worker,
        with: { ...(worker.with as Record<string, string>), job: before.name },
      },
    ],
  });
  workflow.jobs["affected-tooling"] = structuredClone(before);
}

test("maximum non-release runners preserve every original job contract", () => {
  let selectedJobs = 0,
    protectedJobs = 0,
    restoredHostedJobs = 0;
  for (const { specification, before, after } of workflows) {
    const restored = structuredClone(after);
    for (const [id, original] of Object.entries(before.jobs)) {
      const current = restored.jobs[id];
      assert(current, id);
      restoredHostedJobs += Number(restoreMaximumRunner(current, original));
    }
    let restoredWildSteps = 0;
    if (specification.name === "check") {
      restoredWildSteps += Object.values(restored.jobs).reduce(
        (count, job) => count + restorePinnedWildAction(job.steps ?? []),
        0,
      );
      for (const site of ["iteration-plan", "test-report"] as const)
        restoreReviewedCheckout(restored.jobs[site], before.jobs[site], site);
      restoreCoverageBeforeRuntimeProvisioning(restored.jobs["source-coverage"]);
      restoreTestReportComment(restored, before);
      restoreCheckJsCasePacket(restored.jobs["check-js"], before.jobs["check-js"]);
      restoreNodeEngineCasePacket(
        restored.jobs["node-engine-compat"],
        before.jobs["node-engine-compat"],
      );
      restoreContentMapperHelperAbi(restored.jobs["node-engine-compat"]);
    }
    if (specification.name === "pr-source-checks") {
      restoreHistoryToolchainSelection(
        restored.jobs["ci-selection-tests"],
        before.jobs["ci-selection-tests"],
      );
      restoreEmptyPrToolingBuild(restored, before);
      restoreToolingCasePacketJob(restored, before);
      restorePlaygroundCaseReporting(restored, before);
      restoredWildSteps += restorePinnedWildAction(restored.jobs["pr-native-runtime"].steps ?? []);
      restoreNativeRuntimeCasePacket(
        restored.jobs["pr-native-runtime"],
        before.jobs["pr-native-runtime"],
      );
    }
    assert.equal(restoredWildSteps, specification.name === "check" ? 4 : 1);
    for (const id of specification.jobs) {
      const original = before.jobs[id],
        updated = after.jobs[id],
        target = restored.jobs[id];
      assert(original && updated && target, id);
      assert.equal(original["runs-on"], maximumRunner);
      assert.equal(updated["runs-on"], maximumRunner);
      selectedJobs += 1;
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
  assert.equal(selectedJobs, 7);
  assert.equal(protectedJobs, 25);
  assert.equal(restoredHostedJobs, 4);
});

test("empty PR tooling inversion rejects unknown producer and report changes", () => {
  const workflow = workflows.find((entry) => entry.specification.name === "pr-source-checks");
  assert(workflow);
  for (const mutate of [
    (current: Workflow) => {
      current.jobs["affected-tooling-cli"].if = "${{ true }}";
    },
    (current: Workflow) => {
      delete current.jobs["affected-tooling-cli"].needs;
    },
    (current: Workflow) => {
      const step = current.jobs["affected-tooling-report"].steps?.[0];
      assert(step && typeof step.run === "string");
      step.run += "true\n";
    },
    (current: Workflow) => {
      const step = current.jobs["affected-tooling-report"].steps?.[0];
      assert(step);
      step.if = "${{ always() }}";
    },
  ]) {
    const current = structuredClone(workflow.after);
    mutate(current);
    assert.throws(
      () => restoreEmptyPrToolingBuild(current, workflow.before),
      assert.AssertionError,
    );
  }
});

test("history selection refuses missing, duplicate, moved and unknown command changes", () => {
  const workflow = workflows.find((entry) => entry.specification.name === "pr-source-checks");
  assert(workflow);
  const original = workflow.before.jobs["ci-selection-tests"];
  const current = workflow.after.jobs["ci-selection-tests"];
  assert(original && current);
  const restored = structuredClone(current);
  restoreHistoryToolchainSelection(restored, original);
  restored["runs-on"] = maximumRunner;
  assert.deepEqual(restored, original);
  const added = "tests/tooling/native-history-toolchain.test.ts";
  const previous = "tests/tooling/native-history-profile.test.ts";
  for (const mutate of [
    (command: string) => command.replace(" " + added, ""),
    (command: string) => command.replace(added, added + " " + added),
    (command: string) => command.replace(previous + " " + added, added + " " + previous),
    (command: string) => command.replace(added, "tests/tooling/unreviewed-toolchain.test.ts"),
  ]) {
    const changed = structuredClone(current);
    const step = changed.steps?.find(
      (candidate) => candidate.name === "Test CI selection when its inputs change",
    );
    assert(step && typeof step.run === "string");
    step.run = mutate(step.run);
    assert.throws(() => restoreHistoryToolchainSelection(changed, original), assert.AssertionError);
  }
  const extra = structuredClone(current);
  const step = extra.steps?.find(
    (candidate) => candidate.name === "Test CI selection when its inputs change",
  );
  assert(step);
  step.if = "${{ success() }}";
  assert.throws(() => restoreHistoryToolchainSelection(extra, original), assert.AssertionError);
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
        updated = structuredClone(after.jobs[id]);
      assert(original && updated, id);
      if (specification.name === "check" && id === "check-js")
        restoreCheckJsCasePacket(updated, original);
      if (specification.name === "check" && id === "node-engine-compat") {
        restoreNodeEngineCasePacket(updated, original);
        restoreContentMapperHelperAbi(updated);
      }
      if (specification.name === "pr-source-checks" && id === "ci-selection-tests")
        restoreHistoryToolchainSelection(updated, original);
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

test("the exact helper ABI step depends only on the engines selection", () => {
  const current = workflows.find((entry) => entry.specification.name === "check")?.after.jobs[
    "node-engine-compat"
  ];
  assert(current);
  const restored = structuredClone(current);
  restoreContentMapperHelperAbi(restored);
  let vectors = 0;
  for (const event of [...fastEvents, "schedule", "workflow_dispatch"]) {
    for (const dependencies of ["true", "false"]) {
      for (const engines of ["true", "false"]) {
        for (const cancelled of [false, true]) {
          const input = context(event, dependencies, cancelled);
          input.needs["iteration-plan"].outputs.engines = engines;
          assert.equal(evaluate(helperAbiStep.if, input), engines === "true");
          vectors += 1;
        }
      }
    }
  }
  assert.equal(vectors, 40);
});

test("helper ABI inversion rejects missing, duplicate, moved and unknown step changes", () => {
  const current = workflows.find((entry) => entry.specification.name === "check")?.after.jobs[
    "node-engine-compat"
  ];
  assert(current?.steps);
  for (const mutate of [
    (steps: Step[], index: number) => steps.splice(index, 1),
    (steps: Step[], index: number) => steps.splice(index, 0, structuredClone(helperAbiStep)),
    (steps: Step[], index: number) => steps.push(...steps.splice(index, 1)),
    (steps: Step[], index: number) => (steps[index].name = "Unreviewed helper check"),
    (steps: Step[], index: number) => (steps[index].if = "${{ true }}"),
    (steps: Step[], index: number) => (steps[index].run = "true"),
    (steps: Step[], index: number) => (steps[index]["continue-on-error"] = true),
    (steps: Step[], index: number) => (steps[index].env = { UNREVIEWED: "true" }),
  ]) {
    const changed = structuredClone(current);
    const steps = changed.steps!;
    mutate(
      steps,
      steps.findIndex((step) => step.name === helperAbiStep.name),
    );
    assert.throws(() => restoreContentMapperHelperAbi(changed), assert.AssertionError);
  }
});

test("all check-js event classes use the maximum non-release runner", () => {
  const check = workflows.find((entry) => entry.specification.name === "check");
  const selector = check?.after.jobs["check-js"]?.["runs-on"];
  assert(selector);
  assert.equal(selector, maximumRunner);
});

test("test report inversions reject any other change to the report or tooling jobs", () => {
  const find = (name: string) => {
    const entry = workflows.find((candidate) => candidate.specification.name === name);
    assert(entry);
    return entry;
  };
  const check = find("check");
  for (const mutate of [
    (current: Workflow) => {
      current.jobs["test-report"].permissions = { contents: "write" };
    },
    (current: Workflow) => {
      const steps = current.jobs["test-report"].steps;
      assert(steps);
      steps.at(-1)!.run = "true";
    },
    (current: Workflow) => {
      const steps = current.jobs["test-report"].steps;
      assert(steps);
      delete steps.at(-1)!["continue-on-error"];
    },
    (current: Workflow) => {
      current.jobs["test-report"].steps?.push({ name: "Another step", run: "true" });
    },
  ]) {
    const current = structuredClone(check.after);
    mutate(current);
    assert.throws(() => restoreTestReportComment(current, check.before), assert.AssertionError);
  }
  const source = find("pr-source-checks");
  const renamed = structuredClone(source.after);
  const worker = renamed.jobs["affected-tooling"].steps?.at(-1);
  assert(worker);
  worker.with = { ...(worker.with as Record<string, string>), job: "Affected tooling" };
  assert.throws(() => restoreToolingCasePacketJob(renamed, source.before), assert.AssertionError);
});
