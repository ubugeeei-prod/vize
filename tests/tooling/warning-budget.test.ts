import { readToolingProgram } from "./support/program-path.ts";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import { writeFakeCommand } from "./support/fake-command.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const commandPath = "tools/commands/ci/check-warning-budget.ts";

function runWarningBudget(
  body: string,
  args: readonly string[] = [],
): ReturnType<typeof spawnSync<string>> {
  const tempDir = mkdtempSync(path.join(tmpdir(), "vize-warning-budget-"));
  const binDir = path.join(tempDir, "bin");
  try {
    mkdirSync(binDir);
    writeFakeCommand(binDir, "fake-vp", body);
    return spawnSync(process.execPath, [commandPath, "--", "fake-vp", ...args], {
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
  const result = runWarningBudget(readToolingProgram("commands/warning-budget/ansi-summary.cjs"));

  assert.equal(result.status, 1, `${result.stderr}\n${result.stdout}`.trim());
  assert.match(result.stdout, /Found 0 errors and 2 warnings/);
  assert.match(result.stderr, /JS\/TS warning budget is 0 for v1 alpha CI; found 2 warnings\./);
});

test("warning budget fails on unparsed lint warning markers", () => {
  const result = runWarningBudget(
    readToolingProgram("commands/warning-budget/unparsed-warning.cjs"),
  );

  assert.equal(result.status, 1, `${result.stderr}\n${result.stdout}`.trim());
  assert.match(result.stderr, /warn:\s+Lint warnings found/);
  assert.match(
    result.stderr,
    /JS\/TS warning budget is 0 for v1 alpha CI; found unparsed warnings\./,
  );
});

test("warning budget preserves child failures without applying the budget", () => {
  const result = runWarningBudget(readToolingProgram("commands/warning-budget/failed-child.cjs"));

  assert.equal(result.status, 7, `${result.stderr}\n${result.stdout}`.trim());
  assert.match(result.stderr, /Found 0 errors and 2 warnings/);
  assert.doesNotMatch(result.stderr, /JS\/TS warning budget is 0/);
});

test("warning budget accepts zero warnings and forwards both output streams", () => {
  const result = runWarningBudget(readToolingProgram("commands/warning-budget/clean-output.cjs"));

  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, "Found 0 errors and 0 warnings\n");
  assert.equal(result.stderr, "check complete\n");
});

test("warning budget counts summaries across stdout and stderr", () => {
  const result = runWarningBudget(readToolingProgram("commands/warning-budget/mixed-streams.cjs"));

  assert.equal(result.status, 1, result.stderr);
  assert.match(result.stderr, /found 13 warnings\./);
});

test("warning budget preserves summary boundaries between output streams", () => {
  const result = runWarningBudget(readToolingProgram("commands/warning-budget/split-streams.cjs"));

  assert.equal(result.status, 1, result.stderr);
  assert.match(result.stderr, /found 2 warnings\./);
});

test("warning budget reports a missing command and invalid usage", () => {
  const missing = spawnSync(process.execPath, [commandPath, "vize-no-such-command"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  const usage = spawnSync(process.execPath, [commandPath, "--"], {
    cwd: repoRoot,
    encoding: "utf8",
  });

  assert.equal(missing.status, 1, missing.stderr);
  assert.equal(usage.status, 2, usage.stderr);
  assert.match(usage.stderr, /Usage: node .*check-warning-budget\.ts/);
});

test(
  "warning budget treats a signalled child as failure",
  { skip: process.platform === "win32" },
  () => {
    const result = runWarningBudget(readToolingProgram("commands/warning-budget/signal-child.cjs"));

    assert.equal(result.status, 1, result.stderr);
    assert.doesNotMatch(result.stderr, /JS\/TS warning budget is 0/);
  },
);

test(
  "warning budget preserves wide Windows child exit codes",
  { skip: process.platform !== "win32" },
  () => {
    const result = runWarningBudget(readToolingProgram("commands/warning-budget/windows-exit.cjs"));

    assert.equal(result.status, 256, `${result.stderr}\n${result.stdout}`.trim());
  },
);
