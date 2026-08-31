import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const toolPath = path.join(root, "tools", "commands", "ci", "github", "release-platforms.rs");

function releasePlatformPlan(refName: string) {
  const result = spawnSync("rust-script", [toolPath, "print", refName], {
    cwd: root,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}

function applyReleasePlatformCadence(refName: string, packageJsonPath: string) {
  const result = spawnSync("rust-script", [toolPath, "apply-cadence", refName, packageJsonPath], {
    cwd: root,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  return { changed: result.stdout.startsWith("Applied "), skippedTargets: [] };
}

test("release platform plan includes slow targets on fifth minors", () => {
  const plan = releasePlatformPlan("v1.200.0");

  assert.equal(plan.includeSlowPlatforms, true);
  assert.deepEqual(plan.skippedTargets, []);
  assert.ok(plan.cliMatrix.some((platform) => platform.target === "x86_64-apple-darwin"));
  assert.ok(plan.cliMatrix.some((platform) => platform.target === "aarch64-pc-windows-msvc"));
  assert.ok(plan.cliMatrix.some((platform) => platform.target === "x86_64-unknown-linux-musl"));
  assert.ok(plan.cliMatrix.some((platform) => platform.target === "aarch64-unknown-linux-musl"));
  assert.ok(plan.nativeMatrix.some((platform) => platform.target === "x86_64-apple-darwin"));
  assert.ok(plan.nativeMatrix.some((platform) => platform.target === "aarch64-pc-windows-msvc"));
});

test("release platform plan includes slow targets outside fifth minors", () => {
  const plan = releasePlatformPlan("v1.201.0-rc.1");

  assert.equal(plan.includeSlowPlatforms, true);
  assert.deepEqual(plan.skippedTargets, []);
  assert.ok(plan.cliMatrix.some((platform) => platform.target === "x86_64-apple-darwin"));
  assert.ok(plan.cliMatrix.some((platform) => platform.target === "aarch64-pc-windows-msvc"));
  assert.ok(plan.cliMatrix.some((platform) => platform.target === "x86_64-unknown-linux-musl"));
  assert.ok(plan.cliMatrix.some((platform) => platform.target === "aarch64-unknown-linux-musl"));
  assert.ok(plan.nativeMatrix.some((platform) => platform.target === "x86_64-apple-darwin"));
  assert.ok(plan.nativeMatrix.some((platform) => platform.target === "aarch64-pc-windows-msvc"));
});

test("release platform plan builds GNU native packages on the Debian bookworm floor", () => {
  const plan = releasePlatformPlan("v1.201.0");
  const hosts = new Map(plan.nativeMatrix.map(({ host, target }) => [target, host]));

  assert.equal(hosts.get("x86_64-unknown-linux-gnu"), "ubuntu-22.04");
  assert.equal(hosts.get("aarch64-unknown-linux-gnu"), "ubuntu-22.04-arm");
});

test("release platform cadence preserves native manifest entries when all platforms are included", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-platforms-"));
  const packageDir = path.join(tempDir, "npm", "native");
  const packageJsonPath = path.join(packageDir, "package.json");
  const skippedDir = path.join(packageDir, "npm", "darwin-x64");
  const keptDir = path.join(packageDir, "npm", "darwin-arm64");

  try {
    fs.mkdirSync(skippedDir, { recursive: true });
    fs.mkdirSync(keptDir, { recursive: true });
    fs.writeFileSync(
      packageJsonPath,
      `${JSON.stringify(
        {
          name: "@vizejs/native",
          optionalDependencies: {
            "@vizejs/native-darwin-arm64": "1.201.0",
            "@vizejs/native-darwin-x64": "1.201.0",
            "@vizejs/native-win32-arm64-msvc": "1.201.0",
            "@vizejs/native-win32-x64-msvc": "1.201.0",
          },
          napi: {
            targets: [
              "x86_64-apple-darwin",
              "aarch64-apple-darwin",
              "aarch64-pc-windows-msvc",
              "x86_64-pc-windows-msvc",
            ],
          },
        },
        null,
        2,
      )}\n`,
    );

    const result = applyReleasePlatformCadence("v1.201.0", packageJsonPath);
    const updated = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));

    assert.equal(result.changed, false);
    assert.deepEqual(result.skippedTargets, []);
    assert.equal(updated.optionalDependencies["@vizejs/native-darwin-x64"], "1.201.0");
    assert.equal(updated.optionalDependencies["@vizejs/native-win32-arm64-msvc"], "1.201.0");
    assert.equal(updated.optionalDependencies["@vizejs/native-darwin-arm64"], "1.201.0");
    assert.equal(updated.optionalDependencies["@vizejs/native-win32-x64-msvc"], "1.201.0");
    assert.deepEqual(updated.napi.targets, [
      "x86_64-apple-darwin",
      "aarch64-apple-darwin",
      "aarch64-pc-windows-msvc",
      "x86_64-pc-windows-msvc",
    ]);
    assert.equal(fs.existsSync(skippedDir), true);
    assert.equal(fs.existsSync(keptDir), true);
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});
