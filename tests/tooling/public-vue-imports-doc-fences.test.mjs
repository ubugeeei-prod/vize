import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  codeFences,
  createFenceVerifier,
  validateFenceSuccessor,
} from "./support/public-vue-imports-doc-fence-custody.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const source = JSON.parse(
  fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/lint/public-vue-imports-8090/docs-fence-successor.json",
    ),
  ),
);
await test("finite private-import example successor retains all four fences and rejects drift", () => {
  validateFenceSuccessor(source);
  const verify = createFenceVerifier(root);
  for (const filename of [source.originalEntry.path, source.originalEntry.englishPath]) {
    const fences = codeFences(fs.readFileSync(path.join(root, filename), "utf8"));
    assert.equal(verify(source.originalEntry, filename, fences), true);
    for (const mutate of [
      (f) => f.pop(),
      (f) => f.push("```ts\nconst extra = 1;\n```"),
      (f) => f.reverse(),
      (f) => (f[0] = f[0].replace("incremental", "essential")),
      (f) => (f[1] += "\nchanged"),
      (f) => (f[2] = f[2].replace("ref", "privateRef")),
      (f) => (f[3] = f[3].replace("isString", "camelize")),
      (f) => (f[3] = source.beforeCodeFences[3]),
    ]) {
      const changed = structuredClone(fences);
      mutate(changed);
      assert.throws(() => verify(source.originalEntry, filename, changed));
    }
  }
  assert.equal(verify({ ...source.originalEntry, id: "script/other" }, "other", []), false);
  for (const mutate of [
    (s) => (s.unknownField = true),
    (s) => (s.version = 2),
    (s) => (s.originalEntry.id = "script/other"),
    (s) => (s.beforeCodeFences[0] += "changed"),
    (s) => (s.afterCodeFences[0] += "changed"),
    (s) => (s.afterCodeFences[3] += "changed"),
    (s) => s.afterCodeFences.pop(),
    (s) => (s.oldSources[0].source += "changed"),
    (s) => (s.oldSources[0].gitBlob = "0".repeat(40)),
    (s) => (s.originalCustody.sha256 = "0".repeat(64)),
    (s) => (s.originalEntry.wholeCodeSha256 = "0".repeat(64)),
    (s) => (s.change.added = "import { camelize } from '@vue/shared'"),
    (s) => (s.change.changedFences = 2),
  ]) {
    const changed = structuredClone(source);
    mutate(changed);
    assert.throws(() => validateFenceSuccessor(changed));
  }
  assert.throws(() =>
    verify(
      { ...source.originalEntry, supportFacts: ["changed"] },
      source.originalEntry.path,
      source.afterCodeFences,
    ),
  );
  assert.throws(() => verify(source.originalEntry, "unexpected", source.afterCodeFences));
});
