import assert from "node:assert/strict";
// Paths are resolved from the package cwd: the runner virtualizes import.meta.url.
import path from "node:path";

import {
  auditComponentAuthoring,
  formatAuthoringViolations,
} from "@vizejs/ui-tooling/authoring-gate";
import { test } from "vite-plus/test";

test("every component ships a behavior table, interaction tests, and no source-regex behavior assertions", async () => {
  const violations = await auditComponentAuthoring(path.resolve("src"));
  assert.equal(formatAuthoringViolations(violations), "");
  assert.deepEqual(violations, []);
});
