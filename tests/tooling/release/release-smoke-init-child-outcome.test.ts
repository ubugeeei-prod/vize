import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  checkReport,
  projectLocalVizeBin,
  reportedDiagnostics,
} from "../../../tools/support/compat/npm/smoke-release-init-project.mjs";

const REPORT = {
  files: [
    {
      file: "src/main.js",
      diagnostics: ["error:5:7 [TS2322] Type 'number' is not assignable to type 'string'."],
    },
  ],
  errorCount: 1,
  warningCount: 0,
  fileCount: 1,
};
const STDOUT = `${JSON.stringify(REPORT)}\r\n`;
const STDERR = "  authored process boundary\r\n\t";
const ARGS = ["check", "--format", "json", "--quiet"];

type Outcome = {
  pid: number;
  status: number | null;
  signal: string | null;
  stdout: string;
  stderr: string;
  output: (string | null)[];
};
type OutcomeError = Error & { outcome: Outcome };

function childProject(
  t: { after: (fn: () => void) => void },
  termination: string,
  stdout = STDOUT,
) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-init-outcome-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const bin = projectLocalVizeBin(root);
  fs.mkdirSync(path.dirname(bin), { recursive: true });
  fs.writeFileSync(
    bin,
    [
      'const fs = require("node:fs");',
      'fs.writeFileSync("actual-child.json", JSON.stringify({ pid: process.pid, argv: process.argv.slice(2) }));',
      `fs.writeSync(1, ${JSON.stringify(stdout)});`,
      `fs.writeSync(2, ${JSON.stringify(STDERR)});`,
      termination,
      "",
    ].join("\n"),
  );
  return root;
}

function assertRawChild(root: string, outcome: Outcome, stdout = STDOUT) {
  const actual = JSON.parse(fs.readFileSync(path.join(root, "actual-child.json"), "utf8"));
  assert.ok(outcome.pid > 0);
  assert.deepEqual(actual, { pid: outcome.pid, argv: ARGS });
  assert.equal(outcome.stdout, stdout);
  assert.equal(outcome.stderr, STDERR);
  assert.deepEqual(outcome.output, [null, stdout, STDERR]);
}

void test("complete authored diagnostics require a normal exit one and retain the actual child", (t) => {
  const root = childProject(t, "process.exit(1);");
  const result = checkReport(root);
  assertRawChild(root, result.outcome);
  assert.equal(result.status, 1);
  assert.equal(result.signal, null);
  assert.equal(result.outcome.status, 1);
  assert.equal(result.outcome.signal, null);
  assert.deepEqual(result.report, REPORT);
  assert.deepEqual(reportedDiagnostics(result.report, root), REPORT.files);
});

void test("complete identical diagnostics cannot hide an actual fatal child signal", (t) => {
  const root = childProject(t, 'process.kill(process.pid, "SIGTERM");');
  assert.throws(
    () => checkReport(root),
    (rawError: unknown) => {
      assert.ok(rawError instanceof Error);
      const error = rawError as OutcomeError;
      assert.match(error.message, /project-local vize check did not exit normally/);
      assertRawChild(root, error.outcome);
      assert.equal(error.outcome.status, null);
      assert.equal(error.outcome.signal, "SIGTERM");
      assert.equal(error.cause, error.outcome);
      assert.deepEqual(JSON.parse(error.outcome.stdout), REPORT);
      return true;
    },
  );
});

void test("complete identical diagnostics cannot replace exit one with another failure status", (t) => {
  const root = childProject(t, "process.exit(7);");
  assert.throws(
    () => checkReport(root),
    (rawError: unknown) => {
      assert.ok(rawError instanceof Error);
      const error = rawError as OutcomeError;
      assert.match(error.message, /project-local vize check did not exit normally/);
      assertRawChild(root, error.outcome);
      assert.equal(error.outcome.status, 7);
      assert.equal(error.outcome.signal, null);
      assert.deepEqual(JSON.parse(error.outcome.stdout), REPORT);
      return true;
    },
  );
});

void test("a normal malformed JSON child retains untrimmed output beside the parse error", (t) => {
  const stdout = "  {malformed}\r\n";
  const root = childProject(t, "process.exit(1);", stdout);
  assert.throws(
    () => checkReport(root),
    (rawError: unknown) => {
      assert.ok(rawError instanceof Error);
      const error = rawError as OutcomeError;
      assert.match(error.message, /project-local vize check did not produce JSON/);
      assertRawChild(root, error.outcome, stdout);
      assert.equal(error.outcome.status, 1);
      assert.equal(error.outcome.signal, null);
      assert.ok(error.cause instanceof SyntaxError);
      return true;
    },
  );
});
