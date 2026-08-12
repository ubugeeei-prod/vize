import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const registry = JSON.parse(
  fs.readFileSync(path.join(root, "tests/_fixtures/vue-ecosystem-fixtures.json"), "utf8"),
) as {
  projects: Array<{ id: string; revision: string; tsconfig?: string; vueGlobs: string[] }>;
};

test("AIRI fixture pins its complete Vue monorepo surface", () => {
  const project = registry.projects.find((candidate) => candidate.id === "airi");

  assert.ok(project);
  assert.equal(project.revision, "b6011381bc34a6b85ad669363513cb1a2eea6438");
  assert.equal(project.tsconfig, "tsconfig.json");
  assert.deepEqual(project.vueGlobs, [
    "apps/**/*.vue",
    "docs/**/*.vue",
    "packages/**/*.vue",
    "plugins/**/*.vue",
  ]);
});
