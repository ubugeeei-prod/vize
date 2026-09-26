import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("Dump migration rejects alias collisions and preserves wire/API boundaries", () => {
  const result = spawnSync("python3", ["tests/tooling/dump-type-rename.test.py"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr || result.error?.message);
  assert.match(result.stderr, /Ran 10 tests/u);
  assert.match(result.stderr, /\bOK\b/u);
});
