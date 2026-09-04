import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const registryPath = path.join(root, "tests", "_fixtures", "vue-ecosystem-fixtures.json");

const expectedCorpus = {
  "vue-vben-admin": ["playground/src/**/*.vue"],
  hoppscotch: ["packages/hoppscotch-common/src/**/*.vue"],
  "element-plus": ["packages/**/*.vue", "ssr-testing/**/*.vue"],
  "reka-ui": ["packages/core/**/*.vue"],
  primevue: ["packages/primevue/src/**/*.vue"],
  "primevue-volt": ["apps/volt/**/*.vue"],
  "primevue-showcase": ["apps/showcase/**/*.vue"],
} as const;

test("typecheck corpus globs pin the tsconfig-owned Vue sources", () => {
  const registry = JSON.parse(fs.readFileSync(registryPath, "utf8")) as {
    projects: Array<{
      id: string;
      typecheckPerformance?: { corpusGlobs?: string[] };
    }>;
  };
  const actual = Object.fromEntries(
    registry.projects
      .filter((project) => project.typecheckPerformance?.corpusGlobs != null)
      .map((project) => [project.id, project.typecheckPerformance?.corpusGlobs]),
  );
  assert.deepEqual(actual, expectedCorpus);
});
