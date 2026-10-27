import assert from "node:assert/strict";
import { test } from "node:test";
import {
  nativeCheckerImage,
  validateNativeCheckerCapture,
} from "../../tools/benchmarks/scripts/native-checker-cpu-metadata.ts";
void test("captured checker identity rejects unrelated SDKs and rewritten executable paths", () => {
  const original = {
    identity: {
      backend: nativeCheckerImage.path,
      backendSha256: nativeCheckerImage.sha256,
      source: nativeCheckerImage.source,
    },
    binarySha256: nativeCheckerImage.normalSha256,
    command: { args: ["check", "--corsa-path", nativeCheckerImage.path] },
  };
  assert.doesNotThrow(() => validateNativeCheckerCapture(original, nativeCheckerImage.sha256));
  for (const mutation of [
    { ...original, identity: { ...original.identity, backend: "/tmp/unrelated/tsc" } },
    { ...original, identity: { ...original.identity, backendSha256: "0".repeat(64) } },
    { ...original, identity: { ...original.identity, source: "0".repeat(40) } },
    { ...original, binarySha256: "0".repeat(64) },
    { ...original, command: { args: ["check", "--corsa-path", "/tmp/unrelated/tsc"] } },
    { ...original, command: { args: ["check"] } },
    {
      ...original,
      command: {
        args: [
          "check",
          "--corsa-path",
          nativeCheckerImage.path,
          "--corsa-path",
          "/tmp/unrelated/tsc",
        ],
      },
    },
  ])
    assert.throws(() => validateNativeCheckerCapture(mutation, nativeCheckerImage.sha256));
  assert.throws(() => validateNativeCheckerCapture(original, "0".repeat(64)));
});
