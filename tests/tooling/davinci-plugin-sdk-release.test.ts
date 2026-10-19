import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { validateReleaseJobs } from "../../tools/support/compat/github/npm-bootstrap-contract.mjs";
import { bootstrapJobContract } from "../../tools/support/compat/github/npm-bootstrap-packages.mjs";
import { repoRoot } from "./_helpers/tool-commands.ts";
import { readRepoFile } from "./support/github-workflows.ts";
import { requiredSuccessfulReleaseJobs } from "../../tools/support/compat/github/npm-bootstrap-contract.mjs";

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

test("SDK publication uses the built artifact and waits for the smoke", () => {
  const workflow = parse(readRepoFile(".github/workflows/release.yml"));
  const publish = workflow.jobs["publish-npm"];
  assert.ok(publish.needs.includes("smoke"));
  assert.ok(workflow.env.RELEASE_JS_PACKAGES.split(" ").includes(packagePath));
  assert.ok(
    readRepoFile(".github/actions/release-build-packages/action.yml").includes(
      `\n          ${packagePath}\n`,
    ),
  );
  const manifest = JSON.parse(readRepoFile(`${packagePath}/package.json`));
  assert.equal(manifest.dependencies?.["@vizejs/native"], undefined);
  assert.equal(manifest.optionalDependencies?.["@vizejs/native"], undefined);
  assert.deepEqual(manifest.peerDependencies, { "@vizejs/native": ">=0.429.0" });
  assert.deepEqual(manifest.peerDependenciesMeta, { "@vizejs/native": { optional: true } });
  assert.equal(publish.environment, "npm");
  assert.deepEqual(publish.permissions, { contents: "read", "id-token": "write" });
  const bootstrap = parse(readRepoFile(".github/workflows/release-npm-bootstrap.yml"));
  const standalone = bootstrap.jobs.handoff.steps.find((step) =>
    step.run?.includes("davinci-plugin-sdk-package.test.ts"),
  );
  assert.equal(standalone.if, "steps.preflight.outputs.package_path == 'npm/plugin-sdk'");
});

test("the release contracts execute the complete SDK release contracts", () => {
  const carrier = parse(readRepoFile(".github/workflows/release-contracts.yml"));
  const source = "tests/tooling/davinci-plugin-sdk-release.test.ts";
  assert.ok(carrier.on.pull_request.paths.includes(source));
  const steps = carrier.jobs.contracts.steps as Array<{ uses?: string; run?: string }>;
  const policy = steps.findIndex((step) => step.run?.split(/\s+/).includes(source));
  assert.ok(policy >= 0);
  const command = steps[policy]?.run;
  assert.ok(typeof command === "string");
  assert.match(command, /^node --test\s/);
  assert.doesNotMatch(command, /--test-name-pattern|--test-skip-pattern/);
});

test("the release stamp versions the public SDK and keeps its native peer range", () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-sdk-stamp-"));
  const manifest = JSON.parse(readRepoFile(`${packagePath}/package.json`));
  try {
    fs.mkdirSync(path.join(dir, packagePath), { recursive: true });
    fs.mkdirSync(path.join(dir, "npm/cli"));
    fs.writeFileSync(path.join(dir, "Cargo.toml"), '[workspace.package]\nversion = "0.290.0"\n');
    fs.writeFileSync(
      path.join(dir, "Cargo.lock"),
      '[[package]]\nname = "vize"\nversion = "0.290.0"\n',
    );
    fs.writeFileSync(path.join(dir, "npm/cli/package.json"), '{\n  "version": "0.290.0"\n}\n');
    fs.writeFileSync(
      path.join(dir, packagePath, "package.json"),
      `${JSON.stringify({ ...manifest, version: "0.290.0" }, null, 2)}\n`,
    );
    spawnSync("git", ["init", "-q"], { cwd: dir });
    const stamp = spawnSync(
      process.execPath,
      [path.join(repoRoot, "tools/support/release/release/main.ts"), "stamp", "0.291.0"],
      { cwd: dir, encoding: "utf8" },
    );
    assert.equal(stamp.status, 0, stamp.stderr);
    const stamped = JSON.parse(
      fs.readFileSync(path.join(dir, packagePath, "package.json"), "utf8"),
    );
    assert.equal(stamped.version, "0.291.0");
    assert.equal(stamped.private, undefined);
    assert.equal(stamped.peerDependencies["@vizejs/native"], ">=0.429.0");
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});
