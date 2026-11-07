import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import YAML from "yaml";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const actionRoot = path.join(root, ".github/actions/npm-bootstrap-auth");
const read = (...parts: string[]) => fs.readFileSync(path.join(root, ...parts), "utf8");

interface Step {
  id?: string;
  uses?: string;
  with?: Record<string, string>;
}

interface Workflow {
  jobs: Record<string, { "runs-on"?: string; steps?: Step[] }>;
}

test("npm resume confines bootstrap authentication to a post-cleaned first-party action", () => {
  const workflow = YAML.parse(read(".github/workflows/release.yml")) as Workflow;
  const publishJob = workflow.jobs["publish-npm"];
  const steps = publishJob?.steps ?? [];
  const bootstrap = steps.find((step) => step.uses === "./.github/actions/npm-bootstrap-auth");
  const publishIndex = steps.findIndex((step) => step.id === "publish");
  const bootstrapIndex = steps.indexOf(bootstrap as Step);
  assert.deepEqual(bootstrap?.with, {
    token: "${{ inputs.npm_resume_run != '' && secrets.NPM_BOOTSTRAP_TOKEN || '' }}",
  });
  assert.equal(publishJob?.["runs-on"], "ubuntu-24.04");
  assert.ok(bootstrapIndex > 0 && bootstrapIndex < publishIndex);

  const action = YAML.parse(read(".github/actions/npm-bootstrap-auth/action.yml"));
  assert.deepEqual(action.runs, {
    using: "node24",
    main: "main.ts",
    post: "post.ts",
    "post-if": "always()",
  });
});

test("bootstrap auth is private, runner-owned, silent, and removed by the post action", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-npm-bootstrap-"));
  try {
    const githubEnv = path.join(temporary, "env");
    const githubState = path.join(temporary, "state");
    fs.writeFileSync(githubEnv, "");
    fs.writeFileSync(githubState, "");
    const token = "fixture-secret-that-must-not-be-logged";
    const main = spawnSync(process.execPath, [path.join(actionRoot, "main.ts")], {
      encoding: "utf8",
      env: {
        ...process.env,
        INPUT_TOKEN: token,
        RUNNER_TEMP: temporary,
        GITHUB_ENV: githubEnv,
        GITHUB_STATE: githubState,
      },
    });
    assert.equal(main.status, 0, main.stderr);
    assert.equal(`${main.stdout}${main.stderr}`.includes(token), false);
    const authFile = path.join(temporary, "npm-bootstrap.npmrc");
    if (process.platform !== "win32") assert.equal(fs.statSync(authFile).mode & 0o777, 0o600);
    assert.equal(fs.readFileSync(authFile, "utf8"), `//registry.npmjs.org/:_authToken=${token}\n`);
    assert.equal(fs.readFileSync(githubEnv, "utf8"), `NPM_CONFIG_USERCONFIG=${authFile}\n`);
    assert.equal(fs.readFileSync(githubState, "utf8"), `auth=${authFile}\n`);

    const post = spawnSync(process.execPath, [path.join(actionRoot, "post.ts")], {
      encoding: "utf8",
      env: { ...process.env, STATE_auth: authFile, RUNNER_TEMP: temporary },
    });
    assert.equal(post.status, 0, post.stderr);
    assert.equal(fs.existsSync(authFile), false);
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
