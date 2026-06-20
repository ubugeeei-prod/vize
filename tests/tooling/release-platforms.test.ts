import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  applyReleasePlatformCadence,
  releasePlatformPlan,
} from "../../tools/github/release-platforms.mjs";

test("release platform cadence keeps slow targets every fifth minor", () => {
  const plan = releasePlatformPlan("v1.200.0");

  assert.equal(plan.includeSlowPlatforms, true);
  assert.ok(plan.cliMatrix.some((platform) => platform.target === "x86_64-apple-darwin"));
  assert.ok(plan.cliMatrix.some((platform) => platform.target === "aarch64-pc-windows-msvc"));
  assert.ok(plan.nativeMatrix.some((platform) => platform.target === "x86_64-apple-darwin"));
  assert.ok(plan.nativeMatrix.some((platform) => platform.target === "aarch64-pc-windows-msvc"));
});

test("release platform cadence skips slow targets outside fifth minors", () => {
  const plan = releasePlatformPlan("v1.201.0-rc.1");

  assert.equal(plan.includeSlowPlatforms, false);
  assert.deepEqual(plan.skippedTargets, ["x86_64-apple-darwin", "aarch64-pc-windows-msvc"]);
  assert.ok(!plan.cliMatrix.some((platform) => platform.target === "x86_64-apple-darwin"));
  assert.ok(!plan.cliMatrix.some((platform) => platform.target === "aarch64-pc-windows-msvc"));
  assert.ok(!plan.nativeMatrix.some((platform) => platform.target === "x86_64-apple-darwin"));
  assert.ok(!plan.nativeMatrix.some((platform) => platform.target === "aarch64-pc-windows-msvc"));
});

test("release platform cadence removes skipped native manifest entries", () => {
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

    assert.equal(result.changed, true);
    assert.equal(updated.optionalDependencies["@vizejs/native-darwin-x64"], undefined);
    assert.equal(updated.optionalDependencies["@vizejs/native-win32-arm64-msvc"], undefined);
    assert.equal(updated.optionalDependencies["@vizejs/native-darwin-arm64"], "1.201.0");
    assert.equal(updated.optionalDependencies["@vizejs/native-win32-x64-msvc"], "1.201.0");
    assert.deepEqual(updated.napi.targets, ["aarch64-apple-darwin", "x86_64-pc-windows-msvc"]);
    assert.equal(fs.existsSync(skippedDir), false);
    assert.equal(fs.existsSync(keptDir), true);
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});
