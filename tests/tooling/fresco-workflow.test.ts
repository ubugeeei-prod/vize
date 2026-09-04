import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import { hostedOrBlacksmithExact, readRepoFile } from "./support/github-workflows.ts";

const PLATFORM_MATRIX = [
  [hostedOrBlacksmithExact("ubuntu-24.04"), "linux-x64-gnu"],
  [hostedOrBlacksmithExact("macos-15"), "darwin-arm64"],
  [hostedOrBlacksmithExact("windows-2025"), "win32-x64-msvc"],
] as const;

const RELEVANT_PATHS = [
  '".github/workflows/fresco.yml"',
  '"package.json"',
  '"Cargo.lock"',
  '"Cargo.toml"',
  '"crates/vize_fresco/**"',
  '"npm/fresco/**"',
  '"npm/fresco-native/**"',
  '"pnpm-lock.yaml"',
  '"pnpm-workspace.yaml"',
];

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

test("fresco workflow runs on fresco source changes across both trigger events", () => {
  const workflow = readRepoFile(".github", "workflows", "fresco.yml");

  for (const event of ["pull_request", "push"]) {
    const trigger = workflow.slice(workflow.indexOf(`\n  ${event}:\n`));
    assert.match(trigger, /branches: \[main\]\n\s+paths:/, `${event} must filter paths`);
  }
  for (const relevantPath of RELEVANT_PATHS) {
    const pattern = new RegExp(`- ${relevantPath.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}`, "g");
    assert.equal(
      [...workflow.matchAll(pattern)].length,
      2,
      `${relevantPath} must gate both pull_request and push`,
    );
  }
  assert.match(workflow, /\n  workflow_dispatch:\n/);
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
