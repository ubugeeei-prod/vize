import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { hostedOrBlacksmithExact, readRepoFile } from "./support/github-workflows.ts";
import {
  planAffectedJs,
  readJsWorkspace,
} from "../../tools/support/compat/github/plan-affected-js.ts";

const PLATFORM_MATRIX = [
  [hostedOrBlacksmithExact("ubuntu-24.04"), "linux-x64-gnu"],
  [hostedOrBlacksmithExact("macos-15"), "darwin-arm64"],
  [hostedOrBlacksmithExact("windows-2025"), "win32-x64-msvc"],
] as const;

type WorkflowJob = {
  name?: string;
  "runs-on"?: string;
  "timeout-minutes"?: number;
  env?: Record<string, unknown>;
  strategy?: {
    "fail-fast"?: boolean;
    matrix?: {
      platform?: Array<{ runner?: string; target?: string }>;
    };
  };
  steps?: WorkflowStep[];
};

type WorkflowStep = {
  if?: string;
  "continue-on-error"?: boolean;
  name?: string;
  run?: string;
  uses?: string;
  with?: Record<string, unknown>;
};

function assertPlatformMatrix(job: WorkflowJob): void {
  const platforms = job.strategy?.matrix?.platform ?? [];
  assert.equal(platforms.length, PLATFORM_MATRIX.length);

  for (const [runnerPattern, target] of PLATFORM_MATRIX) {
    const platform = platforms.find((candidate) => candidate.target === target);
    assert.ok(platform, `missing platform target ${target}`);
    assert.match(platform.runner ?? "", runnerPattern);
  }
}

function assertLfCheckout(job: WorkflowJob): void {
  // Windows runner images enable autocrlf globally; a CRLF checkout makes the
  // formatter flag every file, so both lanes normalize before checking out.
  const steps = job.steps ?? [];
  const lfStep = steps.findIndex((step) => step.run === "git config --global core.autocrlf false");
  const checkout = steps.findIndex((step) => step.uses?.startsWith("actions/checkout@"));
  assert.notEqual(lfStep, -1, "missing Windows LF checkout step");
  assert.notEqual(checkout, -1, "missing checkout step");
  assert.ok(lfStep < checkout, "LF normalization must precede checkout");
}

function workflowJob(parsed: { jobs?: Record<string, WorkflowJob> }, name: string): WorkflowJob {
  const job = parsed.jobs?.[name];
  assert.ok(job, `missing ${name} job`);
  return job;
}

function workflowStep(job: WorkflowJob, name: string): WorkflowStep {
  const step = job.steps?.find((candidate) => candidate.name === name);
  assert.ok(step, `missing ${name}`);
  return step;
}

function workflowUsesStep(job: WorkflowJob, uses: string): WorkflowStep {
  const step = job.steps?.find((candidate) => candidate.uses?.startsWith(uses));
  assert.ok(step, `missing ${uses} step`);
  return step;
}

test("fresco platform suites run nightly or by explicit dispatch", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "fresco.yml")) as {
    on: Record<string, unknown>;
  };
  assert.equal(Object.hasOwn(workflow.on, "push"), false);
  assert.equal(Object.hasOwn(workflow.on, "pull_request"), false);
  assert.deepEqual(workflow.on.schedule, [{ cron: "11 6 * * *" }]);
  assert.equal(Object.hasOwn(workflow.on, "workflow_dispatch"), true);
});

test("repeated Fresco and storage measurements stay outside ordinary PR and queue checks", () => {
  const fresco = parse(readRepoFile(".github", "workflows", "fresco-dimension-bench.yml"));
  assert.deepEqual(Object.keys(fresco.on).sort(), ["schedule", "workflow_dispatch"]);
  assert.deepEqual(fresco.on.schedule, [{ cron: "37 6 * * *" }]);
  assert.deepEqual(fresco.jobs.dimensions.strategy.matrix.replica, [1, 2, 3]);
  assert.equal(
    workflowStep(fresco.jobs.dimensions, "Compare actual parsing and 1k/10k node layout workloads")
      .run,
    "cargo test --locked --profile ci-opt -p vize_fresco --lib paired_parse_and_layout -- --ignored --test-threads=1",
  );
  const storage = parse(readRepoFile(".github", "workflows", "string-storage-bench.yml"));
  assert.deepEqual(Object.keys(storage.on), ["workflow_dispatch"]);
  for (const sha of ["base_sha", "head_sha"]) {
    assert.equal(storage.on.workflow_dispatch.inputs[sha].required, true);
    assert.equal(storage.on.workflow_dispatch.inputs[sha].type, "string");
  }
  assert.deepEqual(storage.jobs["bounded-storage"].strategy.matrix.repetition, [1, 2, 3]);
  const validate = workflowStep(
    storage.jobs["bounded-storage"],
    "Validate exact commits and use identical benchmark harness",
  );
  assert.match(validate.run ?? "", /\[\[ "\$HEAD_SHA" == "\$DISPATCH_SHA" \]\]/);
  assert.match(validate.run ?? "", /merge-base --is-ancestor "\$BASE_SHA" "\$HEAD_SHA"/);
});

test("affected Fresco conformance keeps NAPI compilation, strict lints and consumer checks daily", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "pr-js-checks.yml"));
  const job = workflow.jobs["pr-js-package-conformance"];
  const build = workflowStep(job, "Check Fresco native declarations and consumer types");
  const lint = workflowStep(job, "Lint Fresco NAPI call sites and test corpus");
  const selected =
    "${{ inputs.run-js && matrix.surface == 'fresco' && fromJSON(inputs.js-plan).conformance.fresco }}";
  assert.equal(build.if, selected);
  assert.equal(lint.if, selected);
  assert.equal(job["continue-on-error"], undefined);
  assert.equal(build["continue-on-error"], undefined);
  assert.equal(lint["continue-on-error"], undefined);
  assert.equal(
    build.run,
    "vp run --filter './npm/fresco-native' check:generated && vp run --filter './npm/fresco-native' check:types",
  );
  assert.equal(
    lint.run?.trim(),
    [
      "cargo clippy --locked --profile ci -p vize_fresco --features napi --lib -- -D warnings",
      "cargo clippy --locked --profile ci -p vize_fresco --lib --tests -- -D warnings",
    ].join("\n"),
  );
  assert.equal(workflowUsesStep(job, "dtolnay/rust-toolchain@").with?.components, "clippy");
  const workspace = readJsWorkspace();
  assert.equal(
    planAffectedJs(workspace, ["crates/vize_fresco/src/layout/dimensions.rs"], {
      packages: ["vize_fresco"],
      scope: "affected",
    }).conformance.fresco,
    true,
  );
  assert.equal(
    planAffectedJs(workspace, ["docs/fresco/performance-policy.md"]).conformance.fresco,
    false,
  );
});

test("fresco JS lane checks, builds, and tests the package on all three platforms", () => {
  const workflow = readRepoFile(".github", "workflows", "fresco.yml");
  const parsed = parse(workflow) as { jobs?: Record<string, WorkflowJob> };
  const job = workflowJob(parsed, "fresco-js");
  const setup = workflowUsesStep(job, "voidzero-dev/setup-vp@");

  assertPlatformMatrix(job);
  assertLfCheckout(job);
  assert.equal(job["timeout-minutes"], 20);
  assert.equal(job.strategy?.["fail-fast"], false);
  assert.equal(setup?.with?.["node-version-file"], "package.json");
  assert.equal(
    workflowStep(job, "Install Fresco JS dependencies").run,
    "vp install --frozen-lockfile --prefer-offline --filter './npm/fresco...'",
  );
  assert.equal(workflowStep(job, "Check Fresco JS").run, "vp run --filter './npm/fresco' check");
  assert.equal(workflowStep(job, "Build Fresco JS").run, "vp run --filter './npm/fresco' build");
  assert.equal(workflowStep(job, "Test Fresco JS").run, "vp run --filter './npm/fresco' test");
});

test("fresco Rust lane tests vize_fresco with per-platform toolchain and caches", () => {
  const workflow = readRepoFile(".github", "workflows", "fresco.yml");
  const parsed = parse(workflow) as { jobs?: Record<string, WorkflowJob> };
  const job = workflowJob(parsed, "fresco-rust");

  assertPlatformMatrix(job);
  assertLfCheckout(job);
  assert.equal(job["timeout-minutes"], 30);
  assert.equal(job.strategy?.["fail-fast"], false);
  assert.equal(workflowStep(job, "Test vize_fresco").run, "cargo test -p vize_fresco");

  const msvc = workflowStep(job, "Setup MSVC toolchain (Windows)");
  assert.equal(msvc.if, "runner.os == 'Windows'");
  assert.match(msvc.uses ?? "", /^ilammy\/msvc-dev-cmd@[0-9a-f]{40}$/);

  const wild = workflowUsesStep(job, "wild-linker/action@");
  assert.equal(wild.if, "runner.os == 'Linux'");
  assert.match(wild.uses ?? "", /^wild-linker\/action@[0-9a-f]{40}$/);
  assert.equal(wild.with?.["wild-version"], "0.9.0");

  // Linux mounts Blacksmith sticky disks; macOS and Windows fall back to the
  // network Rust cache, mirroring the native-smoke split.
  const stickyCache = workflowUsesStep(job, "./.github/actions/setup-rust-sticky-cache");
  const networkCache = workflowUsesStep(job, "Swatinem/rust-cache@");
  assert.equal(stickyCache.if, "runner.os == 'Linux'");
  assert.equal(networkCache.if, "runner.os != 'Linux'");
  assert.match(networkCache.uses ?? "", /^Swatinem\/rust-cache@[0-9a-f]{40}$/);
});

test("fresco package scripts stay portable across CI shells", () => {
  const manifest = JSON.parse(readRepoFile("npm", "fresco", "package.json")) as {
    scripts: Record<string, string>;
  };

  // cmd.exe passes single quotes through literally, which would hand the test
  // runner a quoted glob that matches nothing. Double quotes group on every
  // shell the matrix uses.
  for (const [name, script] of Object.entries(manifest.scripts)) {
    assert.doesNotMatch(script, /'/, `script ${name} must not rely on single-quoted arguments`);
  }
  assert.match(manifest.scripts.test, /--test "src\/\*\*\/\*\.test\.ts"/);
});
