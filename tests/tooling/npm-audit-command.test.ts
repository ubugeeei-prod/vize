import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { writeFakeCommand } from "./support/fake-command.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const scriptPath = path.join(root, "tools", "commands", "ci", "npm-audit.rs");

test("npm audit command retries transient transport failures", () => {
  const fixture = createFixture("eventual-success");

  const result = runAuditCommand(fixture.binDir, {
    TEST_AUDIT_MODE: "eventual-success",
    TEST_COUNTER_FILE: fixture.counterFile,
    TEST_ARGS_FILE: fixture.argsFile,
  });

  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(readArgs(fixture.argsFile), [
    ["exec", "pnpm", "audit", "--prod", "--audit-level", "moderate"],
    ["exec", "pnpm", "audit", "--prod", "--audit-level", "moderate"],
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

function createFixture(mode: string): { argsFile: string; binDir: string; counterFile: string } {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), `vize-npm-audit-${mode}-`));
  const binDir = path.join(tmpDir, "bin");
  const counterFile = path.join(tmpDir, "count");
  const argsFile = path.join(tmpDir, "args.jsonl");
  fs.mkdirSync(binDir);
  writeFakeCommand(
    binDir,
    "vp",
    `
const fs = require("node:fs");
const countFile = process.env.TEST_COUNTER_FILE;
const argsFile = process.env.TEST_ARGS_FILE;
const count = fs.existsSync(countFile) ? Number(fs.readFileSync(countFile, "utf8")) : 0;
fs.writeFileSync(countFile, String(count + 1));
fs.appendFileSync(argsFile, JSON.stringify(process.argv.slice(2)) + "\\n");
if (process.env.TEST_AUDIT_MODE === "eventual-success" && count > 0) {
  process.exit(0);
}
process.exit(Number(process.env.TEST_AUDIT_STATUS ?? "1"));
`,
  );
  return { argsFile, binDir, counterFile };
}

function runAuditCommand(
  binDir: string,
  env: Record<string, string>,
): ReturnType<typeof spawnSync> {
  return spawnSync("rust-script", [scriptPath], {
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
