import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

const root = path.resolve(import.meta.dirname, "../..");
const read = (name: string) => fs.readFileSync(path.join(root, name), "utf8");

test("declaration role corpus preserves actual Volt, watcher and badge contracts", () => {
  const corpus = JSON.parse(read("tests/_fixtures/lsp-declaration-reference-roles.json"));
  const volt = JSON.parse(read(corpus.voltFixture));
  assert.equal(corpus.voltAmbient, volt.ambient);
  assert.equal(volt.sourceBlob, "76ffa1bb83ee2056c60be853c7300db522bfb6c6");
  assert.equal(volt.originalHoverContains, "const tags: string[]");
  for (const entry of corpus.componentAugmentations) {
    const original = read(entry.sourceFile);
    assert.ok(original.includes(entry.declaration), entry.sourceFile);
    assert.ok(
      original.includes(entry.contract) || original.includes(JSON.stringify(entry.contract)),
      entry.sourceFile,
    );
  }
  assert.deepEqual(
    corpus.scopeControls.map((entry: { id: string }) => entry.id),
    [
      "nested-global-interface-is-not-vue-augmentation",
      "nested-global-value-keeps-its-global-role",
      "actual-runtime-core-component-augmentation",
    ],
  );
  const watcher = read("tests/tooling/lsp-watcher-revalidation.test.ts");
  assert.ok(watcher.includes('include: ["src/**/*"]'));
  assert.ok(watcher.includes('writeGlobalComponentDeclaration(declarationPath, "boolean")'));
  assert.ok(watcher.includes("assert.equal(revalidated.version, 1"));
});
