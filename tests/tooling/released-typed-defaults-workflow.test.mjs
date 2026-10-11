import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { overrides } from "../../tools/support/release/typed_defaults/custody.mjs";

const workflow = parse(
  fs.readFileSync(
    new URL("../../.github/workflows/release-typed-defaults-acceptance.yml", import.meta.url),
    "utf8",
  ),
);
const job = workflow.jobs["public-cli"];

await test("runner-dependent consumer paths are resolved only at a permitted step context", () => {
  // GitHub rejects runner context at jobs.<job_id>.env before scheduling a job.
  for (const value of Object.values(job.env)) assert.doesNotMatch(value, /\$\{\{\s*runner\./);
  assert.equal(Object.keys(workflow.on).join(), "workflow_dispatch");
  assert(job.steps[0].env.PUB_CONSUMER.startsWith("${{ runner.temp }}/"));
  assert(job.steps[0].env.PUBLIC_OUTPUT.startsWith("${{ runner.temp }}/"));
});

await test("the actual first-step shell guard rejects foreign refs and every runtime preload before exporting consumer paths", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "typed-public-shell-guard-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const env = {
    ...process.env,
    GITHUB_REPOSITORY: "ubugeeei-prod/vize",
    GITHUB_REF: "refs/heads/main",
    PUB_CONSUMER: path.join(root, "consumer"),
    PUBLIC_OUTPUT: path.join(root, "evidence"),
  };
  for (const key of overrides) delete env[key];
  function run(patch, index) {
    const file = path.join(root, `github-env-${index}`);
    const result = spawnSync("bash", ["-c", job.steps[0].run], {
      env: { ...env, ...patch, GITHUB_ENV: file },
      cwd: root,
      encoding: "utf8",
      timeout: 10_000,
    });
    assert.equal(result.error, undefined);
    return { ...result, exported: fs.existsSync(file) ? fs.readFileSync(file, "utf8") : "" };
  }
  const valid = run({}, "valid");
  assert.equal(valid.status, 0, valid.stderr);
  assert.equal(
    valid.exported,
    `PUB_CONSUMER=${env.PUB_CONSUMER}\nPUBLIC_OUTPUT=${env.PUBLIC_OUTPUT}\n`,
  );
  const invalid = [
    { GITHUB_REF: "refs/heads/untrusted" },
    { GITHUB_REPOSITORY: "foreign/vize" },
    ...overrides.map((key) => ({ [key]: "runtime override" })),
  ];
  for (const [index, patch] of invalid.entries()) {
    const denied = run(patch, index);
    assert.notEqual(denied.status, 0, JSON.stringify(patch));
    assert.equal(denied.exported, "", JSON.stringify(patch));
  }
});
