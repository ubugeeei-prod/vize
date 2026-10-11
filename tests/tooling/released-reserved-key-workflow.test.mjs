import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";
import { overrides } from "../../tools/support/release/typed_defaults/custody.mjs";

const read = (name) => fs.readFileSync(new URL(`../../${name}`, import.meta.url), "utf8");
const original = parse(read(".github/workflows/release-typed-defaults-acceptance.yml"));
const supplement = parse(read(".github/workflows/release-reserved-keys-acceptance.yml"));

await test("reserved443 uses its own manual main observer and retains original392 workflow guards", () => {
  assert.deepEqual(Object.keys(supplement.on), ["workflow_dispatch"]);
  assert.deepEqual(supplement.on, original.on);
  assert.deepEqual(supplement.permissions, original.permissions);
  assert.equal(supplement.concurrency["cancel-in-progress"], false);
  assert.equal(supplement.concurrency.group, "released-reserved-keys-${{ inputs.release_run }}");
  const job = supplement.jobs["public-cli"],
    old = original.jobs["public-cli"];
  assert.equal(job["runs-on"], old["runs-on"]);
  assert.deepEqual(job.env, old.env);
  assert.equal(job.steps[0].run, old.steps[0].run);
  assert.equal(
    job.steps[0].env.PUB_CONSUMER,
    "${{ runner.temp }}/vize-reserved-keys-consumer-${{ github.run_id }}-${{ github.run_attempt }}",
  );
  assert.equal(
    job.steps[0].env.PUBLIC_OUTPUT,
    "${{ runner.temp }}/vize-reserved-keys-evidence-${{ github.run_id }}-${{ github.run_attempt }}",
  );
  assert.deepEqual(job.steps[1], old.steps[1]);
  assert.deepEqual(job.steps[2], old.steps[2]);
  assert.deepEqual(job.steps[3].env, old.steps[3].env);
  assert.equal(job.steps[3].run, "node tools/support/release/typed_defaults/reserved_main.mjs");
  assert.equal(job.steps[4].if, "always()");
  assert.equal(job.steps[4].with["if-no-files-found"], "error");
  assert.equal(job.steps[4].with.path, job.steps[0].env.PUBLIC_OUTPUT);
  assert.equal(job.steps[4].with["include-hidden-files"], true);
  assert.equal(old.steps[3].run, "node tools/support/release/typed_defaults/main.mjs");
});

await test("reserved443 shell refuses foreign refs and all runtime substitutions before Node startup", (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "reserved-workflow-guards-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  function invoke(extra = {}) {
    const output = path.join(directory, "github-env");
    fs.rmSync(output, { force: true });
    // macOS strips DYLD_* before its protected system shell starts. Inject the
    // inert variable in the shell to exercise the same pre-Node guard on both hosts.
    const injected = Object.keys(extra)
      .filter((key) => overrides.includes(key))
      .map((key) => `export ${key}=substituted\n`)
      .join("");
    const result = spawnSync(
      "/bin/bash",
      ["-c", injected + supplement.jobs["public-cli"].steps[0].run],
      {
        encoding: "utf8",
        env: {
          GITHUB_REPOSITORY: "ubugeeei-prod/vize",
          GITHUB_REF: "refs/heads/main",
          PUB_CONSUMER: "/tmp/consumer",
          PUBLIC_OUTPUT: "/tmp/evidence",
          GITHUB_ENV: output,
          ...extra,
        },
      },
    );
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    return { result, written: fs.existsSync(output) ? fs.readFileSync(output, "utf8") : null };
  }
  const valid = invoke();
  assert.equal(valid.result.status, 0);
  assert.equal(valid.written, "PUB_CONSUMER=/tmp/consumer\nPUBLIC_OUTPUT=/tmp/evidence\n");
  for (const change of [
    { GITHUB_REPOSITORY: "foreign/vize" },
    { GITHUB_REF: "refs/heads/unreviewed" },
    { GITHUB_REF: "refs/pull/8684/merge" },
    ...overrides.map((key) => ({ [key]: "substituted" })),
  ]) {
    const refused = invoke(change);
    assert.notEqual(refused.result.status, 0, Object.keys(change)[0]);
    assert.equal(refused.written, null, "refusal must precede environment publication and Node");
  }
});

await test("the actual reserved entry refuses an unreviewed branch before consumer or packet writes", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "reserved-untrusted-observer-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const env = {
    ...process.env,
    PUBLIC_CUT: "a".repeat(40),
    PUBLIC_HEAD: "c".repeat(40),
    PUBLIC_TAG: "v0.999.0",
    PUBLIC_SOURCE_PR: "123",
    PUBLIC_RELEASE_RUN: "456",
    GITHUB_REPOSITORY: "ubugeeei-prod/vize",
    GITHUB_REF: "refs/heads/unreviewed",
    RUNNER_TEMP: root,
    PUB_CONSUMER: path.join(root, "consumer"),
    PUBLIC_OUTPUT: path.join(root, "evidence"),
  };
  for (const key of overrides) delete env[key];
  const run = spawnSync(
    process.execPath,
    [
      fileURLToPath(
        new URL("../../tools/support/release/typed_defaults/reserved_main.mjs", import.meta.url),
      ),
    ],
    { env, encoding: "utf8", timeout: 10_000 },
  );
  assert.equal(run.error, undefined);
  assert.equal(run.signal, null);
  assert.equal(run.status, 1);
  assert.match(run.stderr, /reviewed main observer required|actual hosted GNU consumer required/);
  assert.equal(run.stdout, "");
  assert.equal(fs.existsSync(env.PUB_CONSUMER), false);
  assert.equal(fs.existsSync(env.PUBLIC_OUTPUT), false);
});
