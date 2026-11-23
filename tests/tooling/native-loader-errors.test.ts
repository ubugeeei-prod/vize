import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  assertSucceeded,
  currentTargetPackageName,
  installNativeTarget,
  installPackedNativePackage,
  runNativeLoaderProbe,
} from "./support/native-loader-package.ts";

function isolatedNativeLoaderEnv(extra: NodeJS.ProcessEnv = {}): NodeJS.ProcessEnv {
  const env = { ...process.env };
  for (const key of Object.keys(env)) {
    const normalized = key.toLowerCase();
    if (normalized === "init_cwd" || normalized === "node_path" || normalized.startsWith("npm_")) {
      Reflect.deleteProperty(env, key);
    }
  }
  Reflect.deleteProperty(env, "NAPI_RS_FORCE_WASI");
  Reflect.deleteProperty(env, "NAPI_RS_NATIVE_LIBRARY_PATH");
  Reflect.deleteProperty(env, "VIZE_ALLOW_NATIVE_VERSION_MISMATCH");
  return { ...env, ...extra };
}

test("packed package root distinguishes missing and incompatible native targets", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-loader-error-"));

  try {
    const env = isolatedNativeLoaderEnv();
    const missingInstallDir = installPackedNativePackage(path.join(tempDir, "missing"), env);
    const missing = runNativeLoaderProbe(
      missingInstallDir,
      "reject-missing.cjs",
      [currentTargetPackageName()],
      env,
    );
    assertSucceeded(missing, "require('@vizejs/native') without target package");
    assert.match(missing.stdout, /reported missing optional dependency/);

    const brokenInstallDir = installPackedNativePackage(path.join(tempDir, "broken"), env);
    installNativeTarget(brokenInstallDir, "incompatible-glibc-binding.cjs");
    const broken = runNativeLoaderProbe(brokenInstallDir, "reject-incompatible-glibc.cjs", [], env);
    assertSucceeded(broken, "require('@vizejs/native') with incompatible target package");
    assert.match(broken.stdout, /surfaced native loader failure/);
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});
