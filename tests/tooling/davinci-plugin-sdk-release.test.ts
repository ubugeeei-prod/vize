import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { validateReleaseJobs } from "../../tools/support/compat/github/npm-bootstrap-contract.mjs";
import { bootstrapJobContract } from "../../tools/support/compat/github/npm-bootstrap-packages.mjs";
import { repoRoot } from "./_helpers/tool-commands.ts";
import { readRepoFile } from "./support/github-workflows.ts";
import { requiredSuccessfulReleaseJobs } from "../../tools/support/compat/github/npm-bootstrap-contract.mjs";
import { runRepositoryGuardFixture } from "./support/release-guard-fixture.ts";

const packagePath = "npm/plugin-sdk";

function sdkReleaseJobs() {
  const contract = bootstrapJobContract(packagePath);
  return [
    ...requiredSuccessfulReleaseJobs.map((name) => ({
      name,
      status: "completed",
      conclusion: "success",
    })),
    ...contract.failedJobs.map((name) => ({ name, status: "completed", conclusion: "failure" })),
    ...contract.skippedJobs.map((name) => ({ name, status: "completed", conclusion: "skipped" })),
    { name: "Release native npm", status: "completed", conclusion: "success" },
  ];
}

test("SDK first-publish recovery accepts only its own failed publish after green release gates", () => {
  const command = path.join(repoRoot, "tools/commands/ci/github/npm-bootstrap-preflight.rs");
  const invoke = (jobs: ReturnType<typeof sdkReleaseJobs>) =>
    spawnSync(
      "rust-script",
      [command, "__contract", "release-jobs", JSON.stringify({ packagePath, jobs })],
      { encoding: "utf8" },
    );
  const accepted = sdkReleaseJobs();
  assert.doesNotThrow(() => validateReleaseJobs(accepted, packagePath));
  const result = invoke(accepted);
  assert.equal(result.status, 0, result.stderr);
  for (const changed of [
    accepted.map((job) =>
      job.conclusion === "failure"
        ? { ...job, name: "Release @vizejs/nuxt-lint-config to npm" }
        : job,
    ),
    [...accepted, { name: "Unrelated publish", status: "completed", conclusion: "failure" }],
    accepted.map((job) =>
      job.name === "Release native npm" ? { ...job, conclusion: "skipped" } : job,
    ),
    accepted.map((job) =>
      job.name === "Release native npm" ? { ...job, status: "in_progress" } : job,
    ),
  ]) {
    assert.throws(() => validateReleaseJobs(changed, packagePath));
    assert.notEqual(invoke(changed).status, 0);
  }
});

test("SDK publication uses the built artifact and waits for smoke and release safety", () => {
  const workflow = parse(readRepoFile(".github/workflows/release.yml"));
  const publish = workflow.jobs["release-npm-plugin-sdk"];
  assert.deepEqual(
    new Set(publish.needs),
    new Set(["build-release-packages", "smoke-release-packages", "release-preflight"]),
  );
  const manifest = JSON.parse(readRepoFile(`${packagePath}/package.json`));
  assert.equal(manifest.dependencies?.["@vizejs/native"], undefined);
  assert.equal(manifest.optionalDependencies?.["@vizejs/native"], undefined);
  assert.deepEqual(manifest.peerDependencies, { "@vizejs/native": ">=0.429.0" });
  assert.deepEqual(manifest.peerDependenciesMeta, { "@vizejs/native": { optional: true } });
  assert.equal(publish.environment, "npm");
  assert.deepEqual(publish.permissions, { contents: "read", "id-token": "write" });
  const shared = publish.steps.find(
    (step) => step.uses === "./.github/actions/publish-release-package",
  );
  assert.deepEqual(shared.with, {
    artifact: "release-package-plugin-sdk",
    "package-path": packagePath,
  });
  assert.ok(workflow.jobs["create-github-release"].needs.includes("release-npm-plugin-sdk"));
  const bootstrap = parse(readRepoFile(".github/workflows/release-npm-bootstrap.yml"));
  const standalone = bootstrap.jobs.handoff.steps.find((step) =>
    step.run?.includes("davinci-plugin-sdk-package.test.ts"),
  );
  assert.equal(standalone.if, "steps.preflight.outputs.package_path == 'npm/plugin-sdk'");
});

test("the Rust-ready preflight executes the complete SDK release contracts", () => {
  const carrier = parse(readRepoFile(".github/workflows/release-preflight-contracts.yml"));
  const source = "tests/tooling/davinci-plugin-sdk-release.test.ts";
  assert.ok(carrier.on.pull_request.paths.includes(source));
  const steps = carrier.jobs.contracts.steps as Array<{ uses?: string; run?: string }>;
  const rust = steps.findIndex((step) => step.uses === "./.github/actions/setup-rust-script");
  const policy = steps.findIndex((step) => step.run?.split(/\s+/).includes(source));
  assert.ok(rust >= 0 && policy > rust);
  const command = steps[policy]?.run;
  assert.ok(typeof command === "string");
  assert.match(command, /^node --test\s/);
  assert.doesNotMatch(command, /--test-name-pattern|--test-skip-pattern/);
});

test("release preparation bumps and stages the public SDK using the real manifest sweep", () => {
  const manifestPath = `${packagePath}/package.json`;
  const manifest = JSON.parse(readRepoFile(manifestPath));
  const fixture = runRepositoryGuardFixture({
    branch: "main",
    packageManifests: {
      [manifestPath]: { ...manifest, version: "0.290.0" },
    },
  });
  try {
    assert.equal(fixture.result.status, 0, `${fixture.result.stderr}\n${fixture.result.stdout}`);
    const bumped = JSON.parse(fs.readFileSync(path.join(fixture.tempDir, manifestPath), "utf8"));
    assert.equal(bumped.version, "0.290.1");
    assert.equal(bumped.private, undefined);
    assert.equal(bumped.peerDependencies["@vizejs/native"], ">=0.429.0");
    assert.match(fixture.gitLog, /add .*npm/);
    assert.doesNotMatch(fixture.gitLog, /(?:^|\n)(?:push|tag) /);
  } finally {
    fs.rmSync(fixture.tempDir, { recursive: true, force: true });
  }
});
