import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { verifyStandalonePluginSdk } from "./support/plugin-sdk-package.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

for (const policy of ["default", "offline"] as const) {
  test(`the published SDK installs alone (${policy}), exports every hook and typechecks with the real native peer`, () => {
    const result = verifyStandalonePluginSdk(root, true, undefined, policy);
    assert.equal(result.package, "@vizejs/plugin-sdk");
    assert.equal(result.declarations, true);
    assert.equal(result.nativeInterop, true);
    assert.match(result.runtime, /runtime exports: 8 root, fixes, sandbox and visitor verified/);
  });
}
