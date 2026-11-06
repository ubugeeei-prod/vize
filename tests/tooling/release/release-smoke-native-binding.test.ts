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
  assert.deepEqual(calls[0]?.args.slice(0, 1), ["-e"]);
  assert.equal(calls[0]?.args.at(-1), expectedPackage);
  assert.equal(calls[0]?.options?.cwd, "/fresh/project");
  assert.match(calls[0]?.args[1] ?? "", /assert\.strictEqual\(native, selected/u);

  assertExpectedNativeBinding("/fresh/project", "", () => {
    throw new Error("an unset expected package must skip the native probe");
  });
});
