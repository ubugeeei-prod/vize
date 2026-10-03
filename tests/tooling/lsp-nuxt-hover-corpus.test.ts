import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

const root = path.resolve(import.meta.dirname, "../..");
const read = (name: string) => JSON.parse(fs.readFileSync(path.join(root, name), "utf8"));

test("Nuxt hover corpus retains the original Volt declaration, pin and exact type contract", () => {
  const corpus = read("tests/_fixtures/lsp-nuxt-auto-import-hover.json");
  const registry = read("tests/_fixtures/vue-ecosystem-fixtures.json");
  const project = registry.projects.find((p: { id: string }) => p.id === "primevue-volt");
  assert.ok(project);
  assert.equal(corpus.repository, project.repository);
  assert.equal(corpus.revision, project.revision);
  assert.equal(corpus.componentFile, project.lspAuthoredOracle.templateBinding.file);
  assert.equal(corpus.declaration, project.lspAuthoredOracle.templateBinding.declarationAnchor);
  assert.deepEqual(project.lspAuthoredOracle.templateBinding.hoverContains, [
    corpus.originalHoverContains,
  ]);
  assert.equal(corpus.sourceBlob, "76ffa1bb83ee2056c60be853c7300db522bfb6c6");
  assert.equal(corpus.source.split(corpus.declaration).length, 2);
  assert.doesNotMatch(corpus.source, /\bimport\b/);
  assert.equal(
    corpus.ambient,
    "export {};\ndeclare global { const ref: typeof import('vue')['ref']; }\n",
  );
});
