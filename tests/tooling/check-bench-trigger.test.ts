import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
test("check-bench keeps optional scheduled and manual fail-closed measurements", () => {
  const workflow = fs.readFileSync(path.join(root, ".github/workflows/check-bench.yml"), "utf8");
  assert.match(workflow, /^  workflow_dispatch:$/mu);
  assert.match(workflow, /^  schedule:$/mu);
  assert.doesNotMatch(workflow, /^  pull_request:$/mu);
  assert.match(workflow, /node --test tools\/benchmarks\/scripts\/check-gate-report\.test\.mjs/u);
  assert.match(workflow, /node tools\/benchmarks\/scripts\/generate\.mjs/u);
  assert.match(workflow, /node tools\/benchmarks\/scripts\/check-gate\.mjs/u);
  assert.match(workflow, /--require-vue-tsc/u);
  assert.match(workflow, /if-no-files-found: error/u);
});
