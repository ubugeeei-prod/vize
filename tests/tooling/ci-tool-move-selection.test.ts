import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { planCheckInputs } from "../../tools/support/compat/github/plan-check-inputs.ts";
import { planToolingTests } from "../../tools/support/compat/github/plan-tooling-tests.mjs";

void test("CI commands retain Node compatibility checks after moving directories", () => {
  for (const directory of ["tools/commands/ci", "tools/ci/commands"]) {
    assert.deepEqual(planCheckInputs([`${directory}/github/check-node-engines.ts`]), {
      full: false,
      rust: false,
      dependencies: false,
      engines: true,
      apps: false,
    });
  }
});

void test("shared workflow programs retain their consumers after moving directories", () => {
  for (const directory of [".github/scripts/workflows/shared", "tools/ci/lib/scripts"]) {
    assert.deepEqual(planCheckInputs([`${directory}/rust-host-triple.sed`]), {
      full: false,
      rust: true,
      dependencies: true,
      engines: true,
      apps: true,
    });
  }
});

void test("moved shared scripts and Pkl libraries retain the broad tooling suite", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-ci-tool-move-"));
  const tests = ["tests/tooling/alpha.test.ts", "tests/tooling/beta.test.ts"];
  try {
    for (const directory of ["tests/tooling/davinci", "tests/tooling/release", "tools"]) {
      mkdirSync(join(cwd, directory), { recursive: true });
    }
    for (const file of tests) writeFileSync(join(cwd, file), "");
    for (const file of [
      ".github/scripts/workflows/shared/rust-host-triple.sed",
      "tools/ci/lib/scripts/rust-host-triple.sed",
      "tools/config/pkl/Actions.pkl",
      "tools/ci/lib/Actions.pkl",
      "tools/config/pkl/ShellScript.pkl",
      "tools/ci/lib/ShellScript.pkl",
    ]) {
      assert.deepEqual(planToolingTests([file], { cwd }).tests, tests, file);
    }
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});
