import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import { test } from "node:test";

import { assertExpectedNativeBinding } from "../../../tools/support/compat/npm/smoke-release-init-fresh.mjs";

test("fresh-project native evidence identifies the binding that actually loaded", () => {
  const calls: Array<{
    command: string;
    args: readonly string[];
    options?: { cwd?: string; env?: NodeJS.ProcessEnv };
  }> = [];
  const expectedPackage = "@vizejs/native-linux-x64-musl";
  const token = "0123456789abcdef0123456789abcdef";
  const runCommand = (
    command: string,
    args: readonly string[],
    options?: { cwd?: string; env?: NodeJS.ProcessEnv },
  ) => {
    calls.push({ command, args, options });
    return JSON.stringify({
      schema: "vize.loadedNativeBinding",
      version: 1,
      package: expectedPackage,
      token,
    });
  };

  assertExpectedNativeBinding("/fresh/project", expectedPackage, runCommand);
  assert.equal(calls.length, 1);
  assert.equal(calls[0]?.command, process.execPath);
  const probe = fileURLToPath(
    new URL("../../../tools/support/compat/npm/smoke-loaded-native-package.cjs", import.meta.url),
  );
  assert.deepEqual(calls[0]?.args, [probe, expectedPackage]);
  assert.equal(calls[0]?.options?.cwd, "/fresh/project");
  assert.match(readFileSync(probe, "utf8"), /assert\.strictEqual\(native, selected/u);

  assertExpectedNativeBinding("/fresh/project", "", () => {
    throw new Error("an unset expected package must skip the native probe");
  });
});
