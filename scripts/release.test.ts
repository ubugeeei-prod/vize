import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

const repoRoot = path.resolve(import.meta.dirname, "..");
const cargoToml = readFileSync(path.join(repoRoot, "Cargo.toml"), "utf8");
const currentVersion = cargoToml.match(/^version = "(.+)"$/m)?.[1];

assert.ok(currentVersion, "Failed to read current version from Cargo.toml");

function runBash(script: string) {
  return spawnSync("bash", ["-lc", script], {
    cwd: repoRoot,
    encoding: "utf8",
    stdio: ["pipe", "pipe", "pipe"],
  });
}

test("release script fails clearly when stdin is not interactive", () => {
  const result = runBash("bash ./scripts/release.sh minor");

  assert.equal(result.status, 1);
  assert.equal(
    result.stderr,
    "Error: Confirmation requires an interactive terminal. Re-run with -y to skip the prompt.\n",
  );
  assert.match(
    result.stdout,
    new RegExp(
      `^Current version: ${currentVersion.replaceAll(".", "\\.")}\\nNew version: .+ \\(tag: v.+\\)\\n\\n$`,
    ),
  );
});

test("confirm_release skips prompting when -y is set", () => {
  const result = runBash(
    "source scripts/release.sh; AUTO_CONFIRM=-y; confirm_release; printf 'confirmed\\n'",
  );

  assert.deepEqual(
    {
      status: result.status,
      stdout: result.stdout,
      stderr: result.stderr,
    },
    {
      status: 0,
      stdout: "confirmed\n",
      stderr: "",
    },
  );
});
