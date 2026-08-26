import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { installPnpmDependencies } from "../_helpers/apps.ts";
import { patchPnpmMinimumReleaseAgeExclude } from "../_helpers/pnpm-fixture-config.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}

test("app e2e Playwright config keeps CI runs guarded and debuggable", () => {
  const config = readRepoFile("tests", "app", "playwright.config.ts");

  assert.match(config, /forbidOnly:\s*!!process\.env\.CI/);
  assert.match(config, /retries:\s*process\.env\.CI \? 2 : 0/);
  assert.match(
    config,
    /reporter:\s*process\.env\.CI \? \[\["list"\], \["html", \{ open: "never" \}\]\] : "list"/,
  );
  assert.match(config, /screenshot:\s*"only-on-failure"/);
  assert.match(config, /trace:\s*process\.env\.CI \? "off" : "retain-on-failure"/);
  assert.match(config, /video:\s*process\.env\.CI \? "off" : "retain-on-failure"/);
});

test("external fixture installs use the guarded helper", () => {
  const apps = readRepoFile("tests", "_helpers", "apps.ts");
  const pnpmConfig = readRepoFile("tests", "_helpers", "pnpm-fixture-config.ts");
  assert.doesNotMatch(apps, /execSync\("npx -y pnpm@10 install\b/);
  assert.match(apps, /patchPnpmAgeExclude/);
  assert.match(apps, /@rolldown\/binding-\*/);
  assert.match(pnpmConfig, /minimumReleaseAgeExclude/);
});

test("fixture pnpm workspace patch adds one release-age exclude", () => {
  const tempRoot = fs.mkdtempSync(path.join(root, ".fixture-pnpm-config-"));
  const workspacePath = path.join(tempRoot, "pnpm-workspace.yaml");
  try {
    fs.writeFileSync(
      workspacePath,
      "packages: []\nminimumReleaseAge: 10080\nminimumReleaseAgeExclude:\n  - 'rollup'\n",
    );
    patchPnpmMinimumReleaseAgeExclude(workspacePath, "@rolldown/binding-*");
    patchPnpmMinimumReleaseAgeExclude(workspacePath, "@rolldown/binding-*");

    const source = fs.readFileSync(workspacePath, "utf8");
    assert.match(source, /minimumReleaseAgeExclude:\n  - '@rolldown\/binding-\*'\n  - 'rollup'/);
    assert.equal(source.match(/@rolldown\/binding-\*/g)?.length, 1);
  } finally {
    fs.rmSync(tempRoot, { force: true, recursive: true });
  }
});

test("fixture install child cannot discover or mutate the parent Git repository", () => {
  const tempRoot = fs.mkdtempSync(path.join(root, ".fixture-install-safety-"));
  const fixtureDir = path.join(tempRoot, "fixture");
  const binDir = path.join(tempRoot, "bin");
  const probeScript = path.join(tempRoot, "probe.cjs");
  const probeOutput = path.join(tempRoot, "probe.json");

  try {
    fs.mkdirSync(fixtureDir);
    fs.mkdirSync(binDir);
    writeInstallProbe(probeScript);
    writeNpxShim(binDir, probeScript);

    const before = snapshotParentGitState();
    installPnpmDependencies(fixtureDir, {
      env: {
        ...process.env,
        FIXTURE_INSTALL_PROBE: probeOutput,
        GIT_CEILING_DIRECTORIES: root,
        HUSKY: "1",
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        SKIP_INSTALL_SIMPLE_GIT_HOOKS: "0",
      },
      timeout: 10_000,
    });

    assert.deepEqual(snapshotParentGitState(), before);
    const probe = JSON.parse(fs.readFileSync(probeOutput, "utf8"));
    assert.deepEqual(probe.argv, [
      "-y",
      "pnpm@10",
      "install",
      "--no-frozen-lockfile",
      "--prefer-offline",
    ]);
    assert.equal(probe.env.GIT_CEILING_DIRECTORIES, tempRoot);
    assert.equal(probe.env.HUSKY, "0");
    assert.equal(probe.env.SKIP_INSTALL_SIMPLE_GIT_HOOKS, "1");
    assert.notEqual(probe.gitStatus, 0);
    assert.notEqual(probe.gitTopLevel, root);
  } finally {
    fs.rmSync(tempRoot, { force: true, recursive: true });
  }
});

function snapshotParentGitState() {
  const gitCommonDir = execFileSync(
    "git",
    ["rev-parse", "--path-format=absolute", "--git-common-dir"],
    {
      cwd: root,
      encoding: "utf8",
    },
  ).trim();
  const hooksDir = path.join(gitCommonDir, "hooks");
  return {
    config: fs.readFileSync(path.join(gitCommonDir, "config")),
    hooks: fs.existsSync(hooksDir)
      ? fs
          .readdirSync(hooksDir)
          .sort()
          .map((name) => [name, fs.readFileSync(path.join(hooksDir, name))])
      : [],
  };
}

function writeInstallProbe(probeScript: string): void {
  fs.writeFileSync(
    probeScript,
    `const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const git = spawnSync("git", ["rev-parse", "--show-toplevel"], { encoding: "utf8" });
fs.writeFileSync(process.env.FIXTURE_INSTALL_PROBE, JSON.stringify({
  argv: process.argv.slice(2),
  env: {
    GIT_CEILING_DIRECTORIES: process.env.GIT_CEILING_DIRECTORIES,
    HUSKY: process.env.HUSKY,
    SKIP_INSTALL_SIMPLE_GIT_HOOKS: process.env.SKIP_INSTALL_SIMPLE_GIT_HOOKS,
  },
  gitStatus: git.status,
  gitTopLevel: git.stdout.trim(),
}));
`,
  );
}

function writeNpxShim(binDir: string, probeScript: string): void {
  if (process.platform === "win32") {
    fs.writeFileSync(
      path.join(binDir, "npx.cmd"),
      `@echo off\r\n"${process.execPath}" "${probeScript}" %*\r\n`,
    );
    return;
  }

  const shim = path.join(binDir, "npx");
  fs.writeFileSync(shim, `#!/bin/sh\nexec "${process.execPath}" "${probeScript}" "$@"\n`);
  fs.chmodSync(shim, 0o755);
}
