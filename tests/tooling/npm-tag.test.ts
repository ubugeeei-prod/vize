import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const commandPath = "tools/commands/release/npm/tag.rs";

function runNpmTag(args: readonly string[]) {
  return spawnSync("rust-script", [commandPath, ...args], { cwd: repoRoot, encoding: "utf8" });
}

test("npm tag script maps prerelease versions to npm dist-tags", () => {
  const cases = [
    ["1.2.3-alpha.1", "alpha"],
    ["1.2.3-beta.1", "beta"],
    ["1.2.3-rc.1", "rc"],
    ["1.2.3", "latest"],
  ] as const;

  for (const [version, expected] of cases) {
    const result = runNpmTag([version]);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout.trim(), expected);
  }
});

test("npm tag script prints usage when the version is missing", () => {
  const result = runNpmTag([]);

  assert.equal(result.status, 1, result.stderr);
  assert.match(
    result.stdout,
    /Usage: rust-script tools\/commands\/release\/npm\/tag\.rs <version>/,
  );
});
