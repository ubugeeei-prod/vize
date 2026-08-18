import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import {
  cliReleasePlatforms,
  nativeReleasePlatforms,
} from "../../tools/github/release-platforms.mjs";
import {
  readRepoFile,
  workflowJobBody,
  workflowJobField,
  workflowJobRunsOn,
} from "./support/github-workflows.ts";

const TEMPORARY_HOSTED_RUNNER = "ubuntu-24.04";
const RESTORE_BLACKSMITH_RUNNER = "blacksmith-32vcpu-ubuntu-2404";
const RESTORE_RUNS_ON = `${TEMPORARY_HOSTED_RUNNER} # restore: ${RESTORE_BLACKSMITH_RUNNER}`;

const REMAINING_RELEASE_BLOCKER_FALLBACK_JOBS: Record<string, string[]> = {
  "build-docs.yml": ["build-docs", "build-playground"],
  "check-bench.yml": ["check-bench"],
  "content-mapper-conformance.yml": ["exact-tsgo-project"],
  "deploy-docs.yml": ["deploy"],
  "release-open-vsx.yml": ["release-open-vsx-extension"],
  "vue-benchmarks-replay.yml": ["replay-main", "replay-release"],
};

// Every job the temporary fallback moved off Blacksmith, keyed by workflow. The
// inline `# restore:` comment is the rollback contract for this migration, so
// each job is checked by name: a file-wide label match keeps passing after a
// migrated job silently drops its restore label, and other jobs in these files
// (`ubuntu-latest` planners, release jobs that were always GitHub-hosted) must
// not be annotated.
const HOSTED_FALLBACK_JOBS: Record<string, string[]> = {
  "benchmark.yml": ["pr-benchmark", "pr-benchmark-budget", "pr-benchmark-comment"],
  ...REMAINING_RELEASE_BLOCKER_FALLBACK_JOBS,
  "check.yml": [
    "nix-flake",
    "fmt-rust",
    "check-js",
    "semver-checks",
    "security-audit",
    "node-engine-compat",
    "check-vize-apps",
    "vue-parity",
    "test-scripts",
    "editor-extensions",
    "editor-host-smoke",
    "build-js-packages",
    "test-js-packages",
    "clippy-and-test",
    "coverage",
    "source-coverage",
    "branch-coverage",
    "playground-test",
    "test-report",
    "test-report-comment",
  ],
  "criterion-bench.yml": ["criterion-ab", "dialect-guard"],
  "e2e.yml": ["app-readiness-producer", "app-e2e-producer"],
  "fuzz.yml": ["fuzz"],
  "miri.yml": ["miri"],
  "pkg-pr-new.yml": ["publish-preview"],
  "real-project-matrix.yml": ["real-project-matrix"],
  "release-preflight.yml": ["verify", "validate-crates"],
  "release.yml": [
    "plan-release-platforms",
    "build-editor-extensions",
    "release-vscode-extension",
    "build-release-packages",
    "build-wasm-package",
    "smoke-release-packages",
    "release-crates",
    "create-github-release",
  ],
  "tool-benchmark.yml": [
    "tool-benchmark-impact",
    "tool-benchmark",
    "tool-benchmark-comment",
    "tool-benchmark-commit",
  ],
};

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function activeRunnerLabel(workflow: string, jobName: string): string {
  const runsOn = workflowJobRunsOn(workflow, jobName);
  assert.ok(runsOn, `${jobName} is missing runs-on`);
  return runsOn.split(/\s+#\s*/)[0] ?? runsOn;
}

function parsedTimeoutMinutes(workflow: string, jobName: string): number | undefined {
  const job = (parse(workflow) as { jobs?: Record<string, { "timeout-minutes"?: number }> }).jobs?.[
    jobName
  ];
  assert.ok(job, `${jobName} is missing`);
  return job["timeout-minutes"];
}

function parsedMaxParallel(workflow: string, jobName: string): number | undefined {
  const job = (
    parse(workflow) as { jobs?: Record<string, { strategy?: { "max-parallel"?: number } }> }
  ).jobs?.[jobName];
  assert.ok(job, `${jobName} is missing`);
  return job.strategy?.["max-parallel"];
}

test("temporary hosted runner fallback keeps Blacksmith restore labels per job", () => {
  for (const [workflowName, expectedJobs] of Object.entries(HOSTED_FALLBACK_JOBS)) {
    const source = readRepoFile(".github", "workflows", workflowName);
    const jobs = Object.keys((parse(source) as { jobs?: Record<string, unknown> }).jobs ?? {});

    for (const job of expectedJobs) {
      assert.equal(
        workflowJobRunsOn(source, job),
        RESTORE_RUNS_ON,
        `${workflowName} job ${job} must run on the temporary hosted runner and keep its restore label`,
      );
    }
    assert.deepEqual(
      jobs.filter((job) => workflowJobRunsOn(source, job) === RESTORE_RUNS_ON),
      expectedJobs,
      `${workflowName} annotates a different set of jobs than the hosted fallback covers`,
    );
  }
});

test("remaining release blocker fallback covers each targeted workflow", () => {
  for (const [workflowName, expectedJobs] of Object.entries(
    REMAINING_RELEASE_BLOCKER_FALLBACK_JOBS,
  )) {
    const source = readRepoFile(".github", "workflows", workflowName);
    for (const jobName of expectedJobs) {
      assert.equal(
        activeRunnerLabel(source, jobName),
        TEMPORARY_HOSTED_RUNNER,
        `${workflowName} job ${jobName} must use the temporary hosted runner`,
      );
    }
  }
});

test("temporary native smoke hosted fallback keeps matrix restore labels", () => {
  const source = readRepoFile(".github", "workflows", "native-smoke.yml");
  for (const jobName of ["host-native-smoke", "fresh-install-smoke"]) {
    const job = workflowJobBody(source, jobName);
    for (const [runner, restore, target] of [
      ["ubuntu-24.04", "blacksmith-32vcpu-ubuntu-2404", "linux-x64-gnu"],
      ["ubuntu-24.04-arm", "blacksmith-32vcpu-ubuntu-2404-arm", "linux-arm64-gnu"],
      ["macos-15", "blacksmith-12vcpu-macos-15", "darwin-arm64"],
      ["windows-2025", "blacksmith-32vcpu-windows-2025", "win32-x64-msvc"],
    ] as const) {
      assert.match(
        job,
        new RegExp(
          `runner:\\s*${escapeRegExp(runner)}\\s*# restore: ${escapeRegExp(restore)}\\s*\\n\\s*target:\\s*${escapeRegExp(target)}`,
        ),
        `${jobName} ${target} must keep its Blacksmith restore label`,
      );
    }
  }
});

test("hosted fallback keeps the restore contract on the budgets it widened", () => {
  const source = readRepoFile(".github", "workflows", "e2e.yml");
  assert.equal(
    workflowJobField(source, "app-readiness-producer", "timeout-minutes"),
    `40 # restore: 30 with ${RESTORE_BLACKSMITH_RUNNER}`,
    "the hosted readiness budget must state the Blacksmith value it replaced",
  );

  const realProjectMatrix = readRepoFile(".github", "workflows", "real-project-matrix.yml");
  assert.equal(parsedTimeoutMinutes(realProjectMatrix, "real-project-matrix"), 240);
  const matrixTimeout = workflowJobField(
    realProjectMatrix,
    "real-project-matrix",
    "timeout-minutes",
  );
  assert.ok(matrixTimeout, "missing real-project-matrix timeout-minutes");
  assert.match(
    matrixTimeout,
    /^240\s+# restore:\s+120 with blacksmith-32vcpu-ubuntu-2404$/,
    "the hosted Real Project Matrix budget must keep a Blacksmith restore annotation",
  );
  assert.equal(parsedMaxParallel(realProjectMatrix, "real-project-matrix"), 20);
  assert.match(
    workflowJobBody(realProjectMatrix, "real-project-matrix"),
    /max-parallel:[^\n]*# restore:\s+6 with blacksmith-32vcpu-ubuntu-2404$/m,
    "the hosted Real Project Matrix parallelism must keep a Blacksmith restore annotation",
  );

  const preflight = readRepoFile(".github", "workflows", "release-preflight.yml");
  assert.equal(parsedTimeoutMinutes(preflight, "verify"), 540);
  const preflightTimeout = workflowJobField(preflight, "verify", "timeout-minutes");
  assert.ok(preflightTimeout, "missing verify timeout-minutes");
  assert.match(
    preflightTimeout,
    /^540\s+# restore:\s+120 with blacksmith-32vcpu-ubuntu-2404$/,
    "the hosted release preflight budget must keep a Blacksmith restore annotation",
  );
});

test("check benchmark metadata records the hosted runner that produced it", () => {
  const source = readRepoFile(".github", "workflows", "check-bench.yml");
  const workflow = parse(source) as {
    jobs?: Record<string, { steps?: Array<{ name?: string; run?: string }> }>;
  };
  const gate = workflow.jobs?.["check-bench"]?.steps?.find(
    (step) => step.name === "Run the fail-closed check benchmark gate",
  );
  assert.ok(gate, "missing the check benchmark gate step");
  const activeRun = (gate.run ?? "").replace(/^\s*#.*$/gm, "");
  const runnerLabel = activeRun.match(/--runner-label "([^"]+)"/)?.[1];
  assert.ok(runnerLabel, "missing --runner-label for the check benchmark gate");
  assert.equal(
    runnerLabel,
    activeRunnerLabel(source, "check-bench"),
    "check benchmark metadata must record the runner that produced it",
  );
  assert.ok(
    (gate.run ?? "").includes(`# restore runner-label: ${RESTORE_BLACKSMITH_RUNNER}`),
    "the check benchmark gate must keep the Blacksmith metadata label for restoration",
  );
});

test("tool benchmark metadata records the hosted runner and its restore label", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "tool-benchmark.yml")) as {
    jobs?: Record<string, { steps?: Array<{ name?: string; run?: string }> }>;
  };
  const compare = workflow.jobs?.["tool-benchmark"]?.steps?.find(
    (step) => step.name === "Compare Vize with existing tools",
  );
  assert.ok(compare, "missing the tool benchmark comparison step");
  const run = compare.run ?? "";
  assert.ok(
    run.includes(`--runner-label "${TEMPORARY_HOSTED_RUNNER}"`),
    "the comparison must label its metadata with the runner it actually used",
  );
  assert.ok(
    run.includes(`# --runner-label "${RESTORE_BLACKSMITH_RUNNER}"`),
    "the comparison step must keep the Blacksmith metadata label for restoration",
  );
});

test("release platform metadata keeps the Blacksmith restore mapping on its tables", () => {
  const source = readRepoFile("tools", "github", "release-platforms.mjs");
  const cliTableStart = source.indexOf("export const cliReleasePlatforms = [");
  assert.notEqual(cliTableStart, -1, "missing CLI release platform table");
  assert.ok(
    source.slice(0, cliTableStart).includes("restore macos-15 to blacksmith-12vcpu-macos-15"),
    "the macOS hosted fallback must keep the Blacksmith restore mapping on the platform table",
  );

  // Each migrated host has to remain reachable from a real platform entry;
  // otherwise the mapping above documents a rollback nothing depends on.
  const platforms = [...cliReleasePlatforms, ...nativeReleasePlatforms];
  const cliByTarget = new Map(cliReleasePlatforms.map((platform) => [platform.target, platform]));
  const nativeByTarget = new Map(
    nativeReleasePlatforms.map((platform) => [platform.target, platform]),
  );
  assert.equal(cliByTarget.get("x86_64-apple-darwin")?.host, "macos-15");
  assert.equal(nativeByTarget.get("x86_64-apple-darwin")?.host, "macos-15");
  assert.equal(nativeByTarget.get("aarch64-apple-darwin")?.host, "macos-15");
  for (const host of ["macos-15", "ubuntu-24.04", "ubuntu-24.04-arm"]) {
    assert.ok(
      platforms.some((platform: { host: string }) => platform.host === host),
      `no release platform entry uses the temporary hosted runner ${host}`,
    );
  }
  assert.ok(
    !platforms.some((platform: { host: string }) => platform.host === "macos-15-intel"),
    "MoonBit release scripts do not run on macOS Intel hosted runners",
  );
  for (const restored of [
    "blacksmith-12vcpu-macos-15",
    RESTORE_BLACKSMITH_RUNNER,
    `${RESTORE_BLACKSMITH_RUNNER}-arm`,
  ]) {
    assert.ok(
      !platforms.some((platform: { host: string }) => platform.host === restored),
      `${restored} is still an active release platform host, so the fallback is inconsistent`,
    );
  }
});
