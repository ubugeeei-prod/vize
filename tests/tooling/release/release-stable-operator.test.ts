import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { writeFakeCommand } from "../support/fake-command.ts";
import { readRepoFile } from "../support/github-workflows.ts";

const invocation = parse(
  readRepoFile(".github", "workflows", "release-operator.yml"),
).jobs.operator.steps.find(
  (step: { name?: string }) =>
    step.name === "Authorize the existing maintainer user and invoke the public release command",
);
assert.equal(typeof invocation?.run, "string", "the authorized release command must exist");

test("the actual stable entry binds exact policy/C and refuses injection or mixed protocols before invoking the publisher", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-stable-entry-"));
  const bin = path.join(root, "bin");
  const capture = path.join(root, "argv.json");
  fs.mkdirSync(bin);
  writeFakeCommand(
    bin,
    "gh",
    `
    const a=process.argv.slice(2);
    if(a[0]==='auth')process.exit(0);
    if(a[1]==='user') console.log(a.at(-1)==='.id'?'42':'maintainer');
    else if(a[1].endsWith('/permission'))console.log('maintain'); else process.exit(1);
  `,
  );
  writeFakeCommand(bin, "git", "process.exit(process.argv[2]==='config'?0:1);");
  for (const command of ["vp", "rust-script"])
    writeFakeCommand(
      bin,
      command,
      `require('node:fs').writeFileSync(${JSON.stringify(capture)},JSON.stringify([${JSON.stringify(command)},...process.argv.slice(2)]));`,
    );
  const policy = "a".repeat(40),
    cut = "b".repeat(40);
  const run = (patch: Record<string, string> = {}) =>
    spawnSync("bash", ["-c", invocation.run], {
      cwd: root,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
        GH_TOKEN: "fixture-user",
        GITHUB_ACTOR: "maintainer",
        GITHUB_TRIGGERING_ACTOR: "maintainer",
        GITHUB_REPOSITORY: "ubugeeei-prod/vize",
        GITHUB_SHA: policy,
        RELEASE_OPERATION: "patch",
        RELEASE_SOURCE_CUT: cut,
        RELEASE_SOURCE_PR: "",
        RELEASE_SOURCE_HEAD: "",
        RELEASE_SOURCE_TAG: "",
        RELEASE_RUN: "",
        RELEASE_ORIGINAL_OPERATOR_RUN: "",
        RELEASE_CONSUMER_RECOVERY_RUN: "",
        ...patch,
      },
    });
  try {
    const result = run();
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(JSON.parse(fs.readFileSync(capture, "utf8")), [
      "rust-script",
      "tools/commands/release/pr.rs",
      "start-stable",
      policy,
      cut,
    ]);
    fs.rmSync(capture);
    for (const patch of [
      { RELEASE_SOURCE_CUT: "" },
      { RELEASE_SOURCE_CUT: "main" },
      { RELEASE_SOURCE_CUT: `${cut};touch injected` },
      { GITHUB_SHA: "" },
      { RELEASE_SOURCE_PR: "8541" },
      { RELEASE_SOURCE_HEAD: policy },
      { RELEASE_SOURCE_TAG: "v0.443.1" },
      { RELEASE_RUN: "123" },
      { RELEASE_ORIGINAL_OPERATOR_RUN: "123" },
      { RELEASE_CONSUMER_RECOVERY_RUN: "123" },
      { RELEASE_OPERATION: "start" },
      { RELEASE_OPERATION: "resume", RELEASE_SOURCE_PR: "8541" },
      { RELEASE_OPERATION: "retire", RELEASE_SOURCE_PR: "8541" },
    ]) {
      const refusal = run(patch);
      assert.notEqual(refusal.status, 0, JSON.stringify(patch));
      assert.equal(fs.existsSync(capture), false, JSON.stringify(patch));
      assert.equal(fs.existsSync(path.join(root, "injected")), false);
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
