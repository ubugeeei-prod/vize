import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { runCiBuild } from "../../npm/native/scripts/build-ci.mjs";
import { nativeBuildEnvironment } from "../../npm/native/scripts/native-build-environment.mjs";

void test("build invocations override inherited identity while preserving every other environment field", () => {
  const inherited = {
    PATH: "unchanged-path",
    CUSTOM: "unchanged-value",
    VIZE_NATIVE_BUILD_TOKEN: "stale",
  };
  const first = nativeBuildEnvironment(inherited);
  const second = nativeBuildEnvironment(inherited);
  assert.match(first.VIZE_NATIVE_BUILD_TOKEN, /^[0-9a-f]{32}$/);
  assert.match(second.VIZE_NATIVE_BUILD_TOKEN, /^[0-9a-f]{32}$/);
  assert.notEqual(first.VIZE_NATIVE_BUILD_TOKEN, second.VIZE_NATIVE_BUILD_TOKEN);
  assert.deepEqual({ ...first, VIZE_NATIVE_BUILD_TOKEN: "stale" }, inherited);
  assert.equal(inherited.VIZE_NATIVE_BUILD_TOKEN, "stale");
});

void test("the actual CI producer gives each build a fresh token without adding another native build", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-token-producer-"));
  try {
    const attempts = [];
    const run = (command, args, options) => {
      attempts.push({ command, args, token: options.env.VIZE_NATIVE_BUILD_TOKEN });
      return { status: 0 };
    };
    assert.equal(runCiBuild(directory, { capture: false, run }), 0);
    assert.equal(runCiBuild(directory, { capture: false, run }), 0);
    assert.deepEqual(
      attempts.map(({ command }) => command),
      ["napi", process.execPath, "napi", process.execPath],
    );
    assert.equal(attempts[0].token, attempts[1].token);
    assert.equal(attempts[2].token, attempts[3].token);
    assert.match(attempts[0].token, /^[0-9a-f]{32}$/);
    assert.notEqual(attempts[0].token, attempts[2].token);
    assert.deepEqual(attempts[0].args, attempts[2].args);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
