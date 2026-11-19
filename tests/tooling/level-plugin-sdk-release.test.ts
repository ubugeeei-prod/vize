import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { repoRoot } from "./_helpers/tool-commands.ts";
import { readRepoFile } from "./support/github-workflows.ts";

const packagePath = "npm/plugin-sdk";

test("SDK publication uses the built artifact and waits for the tagged release", () => {
  const workflow = parse(readRepoFile(".github/workflows/release.yml"));
  const publish = workflow.jobs["publish-npm"];
  assert.ok(publish.needs.includes("tag"));
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
  assert.deepEqual(publish.permissions, {
    actions: "read",
    contents: "read",
    "id-token": "write",
  });
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
