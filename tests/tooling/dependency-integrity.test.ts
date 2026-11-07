import assert from "node:assert/strict";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import {
  deepMergePatchHash,
  serializerVersion,
  verifyDependencyIntegrity,
} from "../../tools/support/security/dependency-integrity.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("the installed dependency graph preserves its reviewed integrity boundaries", () => {
  assert.deepEqual(verifyDependencyIntegrity(root), {
    deepMergePatch: deepMergePatchHash,
    mergeChecks: 20,
    serializer: serializerVersion,
  });
});
