import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { failedInstallMessage } from "../../../npm/cli/src/init/install-failure.ts";
import { packageSubdirectoryMessage } from "../../../npm/cli/src/init/package-root.ts";
import { nativeVersionMismatchMessage } from "../../../npm/cli/src/native-version.ts";
import { assertTruncatedNativePackage } from "../../../tools/support/compat/npm/smoke-release-init-failures.ts";
import { readRepoFile } from "../support/github-workflows.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");

test("matching Vize package versions do not block type checking", () => {
  assert.equal(nativeVersionMismatchMessage("0.474.0", "0.474.0"), null);
  assert.equal(
    nativeVersionMismatchMessage("0.1.0", "0.2.0", { VIZE_ALLOW_NATIVE_VERSION_MISMATCH: "1" }),
    null,
  );
});

test("different vize and @vizejs/native versions name both packages and a reinstall", () => {
  const message = nativeVersionMismatchMessage("0.1.0", "0.2.0");
  assert.equal(
    message,
    "vize@0.1.0 and @vizejs/native@0.2.0 are different versions. Reinstall dependencies so type checking uses one Vize release.",
  );
  assert.equal(
    nativeVersionMismatchMessage("", "0.2.0"),
    "vize package version is missing. Reinstall the vize package before type checking.",
  );
  assert.equal(
    nativeVersionMismatchMessage("0.1.0", ""),
    "@vizejs/native package version is missing. Reinstall dependencies so type checking uses one Vize release.",
  );
});

test("a package subdirectory names the nearest package and does not configure type checking", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-subdir-message-"));
  try {
    const app = path.join(root, "packages", "app");
    fs.mkdirSync(path.join(app, "src"), { recursive: true });
    fs.writeFileSync(path.join(root, "package.json"), '{"name":"workspace"}\n');
    fs.writeFileSync(path.join(app, "package.json"), '{"name":"app"}\n');
    assert.equal(packageSubdirectoryMessage(app), null);
    const message = packageSubdirectoryMessage(path.join(app, "src"));
    assert.equal(
      message,
      `No package.json in ${path.resolve(app, "src")}. Run \`vize init\` from ${app} instead of a package subdirectory. Type checking was not configured.`,
    );
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

test("a failed install keeps the package manager output and does not run type checking", () => {
  const message = failedInstallMessage(
    "npm",
    ["install", "-D", "vize"],
    "npm error code ENOTCACHED\n",
    1,
  );
  assert.match(message, /Install command failed \(npm install -D vize\)/);
  assert.match(message, /Type checking was not run/);
  assert.match(message, /ENOTCACHED/);
  const capped = failedInstallMessage("npm", ["install"], "x".repeat(5_000), 1);
  assert.ok(capped.length < 4_200);
  assert.match(capped, /…$/u);
});

test("a truncated native binding names the file and does not load", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-truncated-native-"));
  try {
    for (const name of [
      "native-binding.js",
      "native-binding-identity.js",
      "native-targets.js",
      "package.json",
    ]) {
      fs.copyFileSync(path.join(repoRoot, "npm", "native", name), path.join(root, name));
    }
    assertTruncatedNativePackage(root);
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

test("the packed fresh driver runs the four first-run failure cases", () => {
  const fresh = readRepoFile("tools/support/compat/npm/smoke-release-init-fresh.mjs");
  const failures = readRepoFile("tools/support/compat/npm/smoke-release-init-failures.ts");
  const cli = readRepoFile("npm/cli/src/cli.ts");
  const detect = readRepoFile("npm/cli/src/init/detect.ts");
  const init = readRepoFile("npm/cli/src/init.ts");
  assert.match(fresh, /runInitFailureChecks\(context\)/u);
  assert.match(failures, /assertTruncatedNativePackage/u);
  assert.match(failures, /0\.0\.0-mismatch/u);
  assert.match(failures, /npm_config_offline/u);
  assert.match(failures, /Type checking was not configured/u);
  assert.match(cli, /assertMatchingNativeVersion\(\)/u);
  assert.match(detect, /packageSubdirectoryMessage\(root\)/u);
  assert.match(init, /failedInstallMessage\(/u);
});
