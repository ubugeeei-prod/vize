import { readToolingProgram } from "./support/program-path.ts";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { after, test } from "node:test";
import { fileURLToPath } from "node:url";

import { writeFakeCommand } from "./support/fake-command.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const scriptPath = path.join(root, "tools", "commands", "ci", "npm-audit-retry.ts");
const temporaryDirectories: string[] = [];

after(() => {
  for (const directory of temporaryDirectories)
    fs.rmSync(directory, { recursive: true, force: true });
});

test("npm audit command retries transient transport failures", () => {
  const fixture = createFixture("eventual-success");

  const result = runAuditCommand(fixture.binDir, {
    TEST_AUDIT_MODE: "eventual-success",
    TEST_COUNTER_FILE: fixture.counterFile,
    TEST_ARGS_FILE: fixture.argsFile,
  });

  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(readArgs(fixture.argsFile), [
    ["node", "tools/commands/ci/npm-audit.ts"],
    ["node", "tools/commands/ci/npm-audit.ts"],
  ]);
  assert.match(result.stderr, /attempt 1 failed; retrying/);
});

test("npm audit command preserves the final failure status", () => {
  const fixture = createFixture("always-fail");

  const result = runAuditCommand(fixture.binDir, {
    TEST_AUDIT_MODE: "always-fail",
    TEST_AUDIT_STATUS: "42",
    TEST_COUNTER_FILE: fixture.counterFile,
    TEST_ARGS_FILE: fixture.argsFile,
  });

  assert.equal(result.status, 42, result.stderr);
  assert.equal(readArgs(fixture.argsFile).length, 3);
});

test("npm audit command clamps zero attempts to one", () => {
  const fixture = createFixture("always-fail");
  const result = runAuditCommand(fixture.binDir, {
    TEST_AUDIT_MODE: "always-fail",
    TEST_AUDIT_STATUS: "7",
    TEST_COUNTER_FILE: fixture.counterFile,
    TEST_ARGS_FILE: fixture.argsFile,
    VIZE_NPM_AUDIT_ATTEMPTS: "0",
  });

  assert.equal(result.status, 7, result.stderr);
  assert.equal(readArgs(fixture.argsFile).length, 1);
  assert.doesNotMatch(result.stderr, /retrying/);
});

test("npm audit command uses defaults for invalid attempt counts", () => {
  const fixture = createFixture("always-fail");
  const result = runAuditCommand(fixture.binDir, {
    TEST_AUDIT_MODE: "always-fail",
    TEST_COUNTER_FILE: fixture.counterFile,
    TEST_ARGS_FILE: fixture.argsFile,
    VIZE_NPM_AUDIT_ATTEMPTS: "-1",
  });

  assert.equal(result.status, 1, result.stderr);
  assert.equal(readArgs(fixture.argsFile).length, 3);
});

test(
  "npm audit command retries a signalled child and fails closed",
  { skip: process.platform === "win32" },
  () => {
    const fixture = createFixture("signal");
    const result = runAuditCommand(fixture.binDir, {
      TEST_AUDIT_MODE: "signal",
      TEST_COUNTER_FILE: fixture.counterFile,
      TEST_ARGS_FILE: fixture.argsFile,
    });

    assert.equal(result.status, 1, result.stderr);
    assert.equal(readArgs(fixture.argsFile).length, 3);
  },
);

function createFixture(mode: string): { argsFile: string; binDir: string; counterFile: string } {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), `vize-npm-audit-${mode}-`));
  temporaryDirectories.push(tmpDir);
  const binDir = path.join(tmpDir, "bin");
  const counterFile = path.join(tmpDir, "count");
  const argsFile = path.join(tmpDir, "args.jsonl");
  fs.mkdirSync(binDir);
  writeFakeCommand(binDir, "vp", readToolingProgram("commands/npm-audit-command/vp.js"));
  return { argsFile, binDir, counterFile };
}

function runAuditCommand(
  binDir: string,
  env: Record<string, string>,
): ReturnType<typeof spawnSync> {
  return spawnSync(process.execPath, [scriptPath], {
    cwd: root,
    encoding: "utf8",
    env: {
      ...process.env,
      ...env,
      PATH: `${binDir}${path.delimiter}${process.env.PATH}`,
      VIZE_NPM_AUDIT_RETRY_DELAY_MS: "0",
    },
  });
}

function readArgs(argsFile: string): string[][] {
  return fs
    .readFileSync(argsFile, "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
}
