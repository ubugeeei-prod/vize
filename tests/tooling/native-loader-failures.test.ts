import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { toolingProgram } from "./support/program-path.ts";
import {
  assertSucceeded,
  installNativeTarget,
  installPackedNativePackage,
  runNativeLoaderProbe,
} from "./support/native-loader-package.ts";

test("packed package root surfaces dlopen failures", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-loader-dlopen-"));
  try {
    const installDir = installPackedNativePackage(tempDir);
    installNativeTarget(installDir, toolingProgram("native/broken-glibc-binding.cjs"));
    const rejected = runNativeLoaderProbe(installDir, toolingProgram("native/reject-dlopen.cjs"));
    assertSucceeded(rejected, "require('@vizejs/native') with broken native package");
    assert.match(rejected.stdout, /surfaced dlopen failure/);
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("packed package root surfaces missing internal dependencies", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-loader-internal-"));
  try {
    const installDir = installPackedNativePackage(tempDir);
    installNativeTarget(installDir, toolingProgram("native/missing-internal-binding.cjs"));
    const rejected = runNativeLoaderProbe(
      installDir,
      toolingProgram("native/reject-missing-internal.cjs"),
    );
    assertSucceeded(rejected, "require('@vizejs/native') with missing internal dependency");
    assert.match(rejected.stdout, /surfaced missing internal dependency/);
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});
