import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  assertFixturesPristine,
  cleanFixtures,
  materializedNodeModules,
} from "../../tools/support/compat/levels/lib/corpus-fixture-state.ts";
import { normalizeMachinePaths } from "../../tools/support/compat/levels/lib/corpus-paths.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("corpus baseline and diff commands expose their stable Node interface", () => {
  for (const command of ["corpus-baseline.ts", "corpus-diff.ts"]) {
    const result = spawnSync(
      process.execPath,
      [path.join(repoRoot, "tools", "commands", "levels", command), "--help"],
      { cwd: repoRoot, encoding: "utf8" },
    );
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, new RegExp(`^usage: node tools/commands/levels/${command}`));
    assert.equal(result.stderr, "");
  }
});

test("corpus commands reject unknown arguments before doing corpus work", () => {
  const result = spawnSync(
    process.execPath,
    [path.join(repoRoot, "tools", "commands", "levels", "corpus-baseline.ts"), "--wat"],
    { cwd: repoRoot, encoding: "utf8" },
  );
  assert.equal(result.status, 1);
  assert.equal(result.stdout, "");
  assert.equal(result.stderr, "unknown argument: --wat\n");
});

test("fixture state reports and removes materialized dependency trees", (context) => {
  const root = mkdtempSync(path.join(tmpdir(), "vize-corpus-state-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const fixtureRoot = path.join(root, "tests", "_fixtures", "_git");
  const first = path.join(fixtureRoot, "first", "node_modules");
  const second = path.join(fixtureRoot, "nested", "second", "node_modules");
  mkdirSync(first, { recursive: true });
  mkdirSync(second, { recursive: true });

  assert.deepEqual(materializedNodeModules(root), [
    "tests/_fixtures/_git/first/node_modules",
    "tests/_fixtures/_git/nested/second/node_modules",
  ]);
  assert.throws(
    () => assertFixturesPristine(false, root),
    /2 materialized node_modules directory\(ies\)/,
  );
  assert.doesNotThrow(() => assertFixturesPristine(true, root));
  assert.equal(cleanFixtures(root), 2);
  assert.deepEqual(materializedNodeModules(root), []);
  assert.doesNotThrow(() => assertFixturesPristine(false, root));
});

test("corpus hashes remove machine paths without changing relative diagnostics", () => {
  assert.equal(
    normalizeMachinePaths(
      "project /Users/person/vize/tests/App.vue and /home/runner/work/vize/src/main.ts",
    ),
    "project <abs> and <abs>",
  );
  assert.equal(
    normalizeMachinePaths("src/App.vue /tmp and package/name"),
    "src/App.vue /tmp and package/name",
  );
  assert.equal(normalizeMachinePaths("truncated /home/runner/work/..."), "truncated <abs>...");
});
