import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  isNaturalVOnSource,
  isVOnCorpusInput,
  templateCarrierExtensions,
  V_ON_CORPUS_DIR,
  V_ON_GATE,
} from "../../tools/support/compat/davinci/lib/v-on-corpus.ts";
import { planToolingTests } from "../../tools/support/compat/github/plan-tooling-tests.mjs";

function fixture() {
  const cwd = mkdtempSync(join(tmpdir(), "vize-v-on-selection-"));
  for (const directory of ["tests/tooling", "tests/tooling/davinci", "tests/tooling/release"]) {
    mkdirSync(join(cwd, directory), { recursive: true });
  }
  // No literal imports or repository references: selection must represent the
  // scanner's actual whole-checkout inputs, rather than infer a narrow closure.
  for (const file of [V_ON_GATE, "tests/tooling/unrelated.test.ts"]) {
    writeFileSync(join(cwd, file), "");
  }
  return { cwd, cleanup: () => rmSync(cwd, { recursive: true, force: true }) };
}

test("every source carrier selects the natural corpus guard before merge", () => {
  const f = fixture();
  try {
    for (const extension of templateCarrierExtensions) {
      // Neither new nor deleted source paths have to exist when planning.
      const file = `crates/vize_patina/src/new-or-deleted${extension}`;
      assert.equal(isNaturalVOnSource(file), true);
      assert.deepEqual(planToolingTests([file], { cwd: f.cwd }).tests, [V_ON_GATE]);
    }
    for (const file of ["README.md", "CHANGELOG.md", "docs/new-guide.md", "npm/ui/new.vue"]) {
      assert.deepEqual(planToolingTests([file], { cwd: f.cwd }).tests, [V_ON_GATE]);
    }
  } finally {
    f.cleanup();
  }
});

test("inventory changes select the guard without treating other data as carriers", () => {
  const f = fixture();
  try {
    for (const file of [`${V_ON_CORPUS_DIR}/crates--vize_patina.tsv`, V_ON_GATE]) {
      assert.deepEqual(planToolingTests([file], { cwd: f.cwd }).tests, [V_ON_GATE]);
    }
    assert.equal(isNaturalVOnSource(V_ON_GATE), false, "the scanner excludes its synthetic gate");
    for (const file of ["tests/data.json", "crates/Original.html.txt", "docs/unrelated.tsv"]) {
      assert.equal(isVOnCorpusInput(file), false);
      assert.deepEqual(planToolingTests([file], { cwd: f.cwd }).tests, []);
    }
    assert.deepEqual(
      planToolingTests([], { cwd: f.cwd, tier: "merge" }).tests,
      ["tests/tooling/unrelated.test.ts", V_ON_GATE].sort(),
    );
  } finally {
    f.cleanup();
  }
});

test("the actual Patina event sources select the existing guard in the PR plan", () => {
  for (const file of [
    "crates/vize_patina/src/native/template/context/event/tests.rs",
    "crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers/native/tests.rs",
  ]) {
    const plan = planToolingTests([file]);
    assert.equal(plan.reason, "import closures and named repository paths");
    assert.equal(plan.tests.includes(V_ON_GATE), true, file);
    assert.equal(plan.tests.includes("tests/tooling/lsp-smoke.test.ts"), false);
  }
});
