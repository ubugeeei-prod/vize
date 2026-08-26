import assert from "node:assert/strict";
import { test } from "node:test";

import { hostedOrBlacksmith, readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

const PLATFORM_MATRIX = [
  [hostedOrBlacksmith("ubuntu-24.04"), "linux-x64-gnu"],
  [hostedOrBlacksmith("macos-15"), "darwin-arm64"],
  [hostedOrBlacksmith("windows-2025"), "win32-x64-msvc"],
] as const;

const RELEVANT_PATHS = [
  '".github/workflows/fresco.yml"',
  '".node-version"',
  '"Cargo.lock"',
  '"Cargo.toml"',
  '"crates/vize_fresco/**"',
  '"npm/fresco/**"',
  '"npm/fresco-native/**"',
  '"pnpm-lock.yaml"',
  '"pnpm-workspace.yaml"',
];

function assertPlatformMatrix(job: string): void {
  for (const [runner, target] of PLATFORM_MATRIX) {
    assert.match(job, new RegExp(`runner:\\s*${runner}\\s*\\n\\s+target:\\s*${target}`));
  }
}

function assertLfCheckout(job: string): void {
  // Windows runner images enable autocrlf globally; a CRLF checkout makes the
  // formatter flag every file, so both lanes normalize before checking out.
  const lfStep = job.indexOf("git config --global core.autocrlf false");
  const checkout = job.indexOf("uses: actions/checkout@");
  assert.notEqual(lfStep, -1, "missing Windows LF checkout step");
  assert.ok(lfStep < checkout, "LF normalization must precede checkout");
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
  const job = workflowJobBody(workflow, "fresco-js");

  assertPlatformMatrix(job);
  assertLfCheckout(job);
  assert.match(job, /timeout-minutes:\s*20\b/);
  assert.match(job, /fail-fast:\s*false/);
  assert.match(job, /node-version-file:\s*"\.node-version"/);
  assert.match(
    job,
    /vp install --frozen-lockfile --prefer-offline --filter '\.\/npm\/fresco\.\.\.'/,
  );
  for (const task of ["check", "build", "test"]) {
    assert.match(job, new RegExp(`vp run --filter '\\./npm/fresco' ${task}\\b`));
  }
});

test("fresco Rust lane tests vize_fresco with per-platform toolchain and caches", () => {
  const workflow = readRepoFile(".github", "workflows", "fresco.yml");
  const job = workflowJobBody(workflow, "fresco-rust");

  assertPlatformMatrix(job);
  assertLfCheckout(job);
  assert.match(job, /timeout-minutes:\s*30\b/);
  assert.match(job, /fail-fast:\s*false/);
  assert.match(job, /cargo test -p vize_fresco\b/);
  assert.match(
    job,
    /if:\s*runner\.os == 'Windows'\s*\n\s+uses:\s*ilammy\/msvc-dev-cmd@[0-9a-f]{40}/,
  );
  assert.match(
    job,
    /uses:\s*wild-linker\/action@[0-9a-f]{40}\s*# v0\.9\.0\s*\n\s+if:\s*runner\.os == 'Linux'/,
  );
  // Linux mounts Blacksmith sticky disks; macOS and Windows fall back to the
  // network Rust cache, mirroring the native-smoke split.
  assert.match(
    job,
    /setup-rust-sticky-cache\s*\n\s+if:\s*runner\.os == 'Linux'[\s\S]*Swatinem\/rust-cache@[0-9a-f]{40}\s*# v2\s*\n\s+if:\s*runner\.os != 'Linux'/,
  );
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
