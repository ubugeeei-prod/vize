import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  assertMembershipCorrection,
  assertParity,
  command,
  contractDirectory,
  fixture,
  indexFile,
  object,
  referenceFixtures,
  reportFile,
  root,
  runReporter,
  type Fixture,
  type Oracle,
  type Result,
} from "../_helpers/lint-report-rust-parity.ts";

interface ReviewedCase {
  id: string;
  project: string;
  revision: string;
  source: string;
  finding: { file: string; [key: string]: unknown };
}

interface ScorecardParityOptions {
  compare: (input: Fixture, args?: string[], oracles?: Oracle[]) => Result;
  updatedReporter: () => string;
}

const nodeCommand = path.join(root, `${command}.ts`);

export function registerScorecardParityTests({
  compare,
  updatedReporter,
}: ScorecardParityOptions): void {
  const reviewedValue: unknown = JSON.parse(
    fs.readFileSync(path.join(root, contractDirectory, "contract.json"), "utf8"),
  );
  const reviewedCases = object(reviewedValue).cases;
  assert.ok(Array.isArray(reviewedCases));
  const reviewed = reviewedCases.map((value: unknown): ReviewedCase => {
    const row = object(value);
    for (const key of ["id", "project", "revision", "source"])
      assert.equal(typeof row[key], "string");
    assert.equal(typeof object(row.finding).file, "string");
    return row as unknown as ReviewedCase;
  });
  for (const entry of reviewed) {
    test(`updated real Rust and TS preserve full reviewed case ${entry.id} and missing/duplicate findings`, () => {
      const input = fixture();
      try {
        fs.rmSync(path.join(input.directory, "App.vue"));
        input.write(
          entry.finding.file,
          fs.readFileSync(path.join(root, contractDirectory, entry.source), "utf8"),
        );
        const project = input.project(entry.project, {
          revision: entry.revision,
          vueGlobs: [entry.finding.file],
          expectedVueFileCount: 1,
        });
        const { file, ...message } = entry.finding;
        const expectedNegatives = object(referenceFixtures).reviewedUnrelatedFalseNegatives;
        const falseNegatives =
          object(expectedNegatives)[entry.id] ??
          object(expectedNegatives)[entry.id.replace(/-scorecard$/u, "")];
        assert.ok(Array.isArray(falseNegatives));
        for (const count of [1, 0, 2]) {
          input.configure([project], {
            results: [{ file, messages: Array.from({ length: count }, () => message) }],
          });
          const result = compare(input, input.args, ["updated"]);
          assert.equal(
            result.status,
            count === 0 && falseNegatives.length === 0 ? 0 : 1,
            result.stderr,
          );
          const report = reportFile(result, `${entry.project}-lint-divergence.json`);
          assert.deepEqual(report.divergence.falseNegatives, falseNegatives);
          assert.equal(report.divergence.summary.falseNegativeCount, falseNegatives.length);
          assert.equal(
            report.budget.verdict,
            count > 0 || falseNegatives.length > 0 ? "breached" : "passed",
          );
          assert.equal(report.divergence.referenceCoverageExpectations.length, 0);
          assert.equal(report.divergence.summary.referenceCoverageIntegrityErrorCount, 0);
          assert.equal(report.divergence.summary.falsePositiveCount, count);
          assert.equal(report.budget.maxFalsePositiveCount, 0);
          assert.equal(report.budget.maxFalseNegativeCount, 0);
        }
      } finally {
        input.clean();
      }
    });
  }
  test("scorecard eligibility rejects inconsistent budgets, index totals and exit packets", () => {
    const input = fixture();
    try {
      const entry = reviewed[0];
      fs.rmSync(path.join(input.directory, "App.vue"));
      input.write(
        entry.finding.file,
        fs.readFileSync(path.join(root, contractDirectory, entry.source), "utf8"),
      );
      const project = input.project(entry.project, {
        revision: entry.revision,
        vueGlobs: [entry.finding.file],
        expectedVueFileCount: 1,
      });
      const { file, ...message } = entry.finding;
      input.configure([project], { results: [{ file, messages: [message, message] }] });
      const expected = runReporter(input, updatedReporter(), input.args, "updated"),
        actual = runReporter(input, nodeCommand);
      assertParity(expected, actual, "updated");
      const rejects = (mutate: (result: typeof actual) => void) => {
        const corrupted = structuredClone(actual);
        mutate(corrupted);
        assert.throws(() => assertParity(expected, corrupted, "updated"));
      };
      rejects((result) => {
        const budget = reportFile(result, `${entry.project}-lint-divergence.json`).budget;
        budget.passed = true;
        budget.verdict = "passed";
      });
      rejects((result) => {
        indexFile(result).totals.falsePositiveCount++;
      });
      rejects((result) => {
        reportFile(
          result,
          `${entry.project}-lint-divergence.json`,
        ).divergence.summary.falsePositiveRatio = 0;
      });
      rejects((result) => {
        const name = `${entry.project}-lint-divergence.md`,
          markdown = result.files[name];
        if (typeof markdown !== "string") assert.fail(`${name}: expected Markdown`);
        result.files[name] = markdown.replace("Budget passed: false", "Budget passed: true");
      });
      rejects((result) => {
        result.status = 0;
        result.stderr = "";
      });
    } finally {
      input.clean();
    }
  });
  test("unmeasured foreign reviewed source is scored: explicit correction to updated Rust membership bug", () => {
    const input = fixture();
    try {
      const entry = reviewed[0];
      input.write(
        entry.finding.file,
        fs.readFileSync(path.join(root, contractDirectory, entry.source), "utf8"),
      );
      const { file, ...message } = entry.finding;
      input.configure([input.project(entry.project, { revision: entry.revision })], {
        results: [{ file, messages: [message] }],
      });
      const old = runReporter(input, updatedReporter(), input.args, "updated"),
        actual = runReporter(input, nodeCommand);
      assertMembershipCorrection(old, actual, object(entry));
    } finally {
      input.clean();
    }
  });
}
