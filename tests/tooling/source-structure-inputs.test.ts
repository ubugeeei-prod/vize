import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { planToolingTests } from "../../tools/support/compat/github/plan-tooling-tests.mjs";

const guard = "tests/tooling/source-file-lengths.test.ts";
const unrelated = "tests/tooling/unrelated.test.ts";
const deferred = "tests/tooling/lsp-smoke.test.ts";

function fixture() {
  const cwd = fs.mkdtempSync(path.join(os.tmpdir(), "vize-structure-inputs-"));
  for (const directory of ["tests/tooling", "tests/tooling/davinci", "tests/tooling/release"]) {
    fs.mkdirSync(path.join(cwd, directory), { recursive: true });
  }
  // Deliberately provide no literal imports or named source paths: the real
  // scanner reads the entire checkout, so inference alone cannot establish its inputs.
  for (const file of [guard, unrelated, deferred]) {
    fs.writeFileSync(path.join(cwd, file), "");
  }
  return { cwd, cleanup: () => fs.rmSync(cwd, { recursive: true, force: true }) };
}

for (const [name, changed] of [
  ["benchmark", "crates/vize_canon/benches/partition_weight.rs"],
  ["nested Rust test module", "crates/vize_maestro/src/server/initial_diagnostics/tests.rs"],
  ["ordinary Rust source", "crates/vize_canon/src/lib.rs"],
  ["frontend source", "npm/builder/vite/src/index.ts"],
] as const) {
  test(`the whole-tree structure guard is selected for ${name} changes`, () => {
    const f = fixture();
    try {
      const plan = planToolingTests([changed], { cwd: f.cwd });
      assert.deepEqual(plan.tests, [guard]);
      assert.equal(plan.deferredTests, 1);
    } finally {
      f.cleanup();
    }
  });
}

test("unknown and empty inputs retain the broad PR fallback without deferred scenarios", () => {
  const f = fixture();
  try {
    for (const changed of [[], ["unregistered/input.rs"]]) {
      assert.deepEqual(planToolingTests(changed, { cwd: f.cwd }).tests, [guard, unrelated]);
    }
  } finally {
    f.cleanup();
  }
});

test("the merge tier retains the entire existing suite", () => {
  const f = fixture();
  try {
    const plan = planToolingTests(["crates/vize_canon/benches/partition_weight.rs"], {
      cwd: f.cwd,
      tier: "merge",
    });
    assert.deepEqual(plan.tests, [deferred, guard, unrelated]);
    assert.equal(plan.deferredTests, 0);
  } finally {
    f.cleanup();
  }
});

test("an unregistered prose path does not make the PR suite unconditional", () => {
  const f = fixture();
  try {
    assert.deepEqual(planToolingTests(["CHANGELOG.md"], { cwd: f.cwd }).tests, []);
  } finally {
    f.cleanup();
  }
});
