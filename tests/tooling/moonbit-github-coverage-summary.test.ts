import assert from "node:assert/strict";
import fs, { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { runMoonScript } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

test("github/write_coverage_summary appends the tail of coverage output", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-coverage-summary-"));
  const binDir = path.join(tempDir, "bin");
  const summaryPath = path.join(tempDir, "step-summary.md");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeCommand(
      binDir,
      "cargo",
      [
        "const args = process.argv.slice(2);",
        "if (args.join(' ') !== 'run --profile ci -p vize_test_runner --bin coverage') process.exit(99);",
        "process.stdout.write([",
        "  'Coverage report',",
        "  'line-1',",
        "  'line-2',",
        "  'line-3',",
        "  'line-4',",
        "  'line-5',",
        "  'line-6',",
        "  'line-7',",
        "  'line-8',",
        "].join('\\n'));",
      ].join("\n"),
    );
    writeFileSync(summaryPath, "existing\n");

    const result = runMoonScript("github/write_coverage_summary", [], {
      cwd: tempDir,
      env: {
        GITHUB_STEP_SUMMARY: summaryPath,
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.equal(
      fs.readFileSync(path.join(tempDir, "coverage-report.txt"), "utf8"),
      [
        "Coverage report",
        "line-1",
        "line-2",
        "line-3",
        "line-4",
        "line-5",
        "line-6",
        "line-7",
        "line-8",
      ].join("\n"),
    );
    assert.equal(
      fs.readFileSync(summaryPath, "utf8"),
      [
        "existing",
        "### Coverage Summary",
        "line-2",
        "line-3",
        "line-4",
        "line-5",
        "line-6",
        "line-7",
        "line-8",
      ].join("\n") + "\n",
    );
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("github/write_coverage_summary preserves failing coverage output before returning non-zero", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-coverage-summary-failure-"));
  const binDir = path.join(tempDir, "bin");
  const summaryPath = path.join(tempDir, "step-summary.md");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeCommand(
      binDir,
      "cargo",
      [
        "const args = process.argv.slice(2);",
        "if (args.join(' ') !== 'run --profile ci -p vize_test_runner --bin coverage') process.exit(99);",
        "process.stdout.write([",
        "  'Vue Compiler Coverage Report',",
        "  '----------------------------',",
        "  'TOTAL:  502/594 (84.5%)',",
        "  '92 tests failed',",
        "].join('\\n'));",
        "process.exit(7);",
      ].join("\n"),
    );
    writeFileSync(summaryPath, "existing\n");

    const result = runMoonScript("github/write_coverage_summary", [], {
      cwd: tempDir,
      env: {
        GITHUB_STEP_SUMMARY: summaryPath,
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
      },
    });

    assert.equal(result.status, 7, `${result.stderr}\n${result.stdout}`.trim());
    assert.equal(
      fs.readFileSync(path.join(tempDir, "coverage-report.txt"), "utf8"),
      [
        "Vue Compiler Coverage Report",
        "----------------------------",
        "TOTAL:  502/594 (84.5%)",
        "92 tests failed",
      ].join("\n"),
    );
    assert.equal(
      fs.readFileSync(summaryPath, "utf8"),
      [
        "existing",
        "### Coverage Summary",
        "Vue Compiler Coverage Report",
        "----------------------------",
        "TOTAL:  502/594 (84.5%)",
        "92 tests failed",
      ].join("\n") + "\n",
    );
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
