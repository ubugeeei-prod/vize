import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import { writeFakeCommand } from "./support/fake-command.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const commandPath = "tools/commands/ci/check-warning-budget.rs";

function runWarningBudget(
  body: string,
  args: readonly string[] = [],
): ReturnType<typeof spawnSync<string>> {
  const tempDir = mkdtempSync(path.join(tmpdir(), "vize-warning-budget-"));
  const binDir = path.join(tempDir, "bin");
  try {
    mkdirSync(binDir);
    writeFakeCommand(binDir, "fake-vp", body);
    return spawnSync("rust-script", [commandPath, "--", "fake-vp", ...args], {
      cwd: repoRoot,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
      },
    });
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
}

test("warning budget fails on parsed JS/TS warning summaries", () => {
  const result = runWarningBudget(`console.log("\\x1b[33mFound 0 errors and 2 warnings\\x1b[0m");`);

  assert.equal(result.status, 1, `${result.stderr}\n${result.stdout}`.trim());
  assert.match(result.stdout, /Found 0 errors and 2 warnings/);
  assert.match(result.stderr, /JS\/TS warning budget is 0 for v1 alpha CI; found 2 warnings\./);
});

test("warning budget fails on unparsed lint warning markers", () => {
  const result = runWarningBudget(`console.error("warn:    Lint warnings found");`);

  assert.equal(result.status, 1, `${result.stderr}\n${result.stdout}`.trim());
  assert.match(result.stderr, /warn:\s+Lint warnings found/);
  assert.match(
    result.stderr,
    /JS\/TS warning budget is 0 for v1 alpha CI; found unparsed warnings\./,
  );
});

test("warning budget preserves child failures without applying the budget", () => {
  const result = runWarningBudget(
    `console.error("Found 0 errors and 2 warnings");
process.exit(7);`,
  );

  assert.equal(result.status, 7, `${result.stderr}\n${result.stdout}`.trim());
  assert.match(result.stderr, /Found 0 errors and 2 warnings/);
  assert.doesNotMatch(result.stderr, /JS\/TS warning budget is 0/);
});
