import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { encodeFrames, decodeFrames } from "../../tools/support/ci/lint-range-capture/codec.ts";
import {
  validateReport,
  validateSummary,
} from "../../tools/support/ci/lint-range-capture/reports.ts";
import { projects } from "../../tools/support/ci/lint-range-capture/projects.ts";
import { originalExit, reporterArgs } from "../../tools/support/ci/lint-range-capture/run.ts";

const expected = "a".repeat(40);
const makeReport = () => ({
  schema: "vize.fixtureLintDivergenceRun",
  version: 2,
  project: projects[0].id,
  revision: projects[0].revision,
  evidence: { commitSha: expected },
  preset: "ecosystem",
  baseline: { version: "10.9.2" },
  files: { comparedCount: 1 },
  divergence: {
    summary: { baselineInvalidRangeCount: 1 },
    baselineInvalidRanges: [
      {
        file: "src/App.vue",
        finding: {
          ruleId: "vue/no-v-html",
          column: 0,
          message: "original 漢字",
          nested: { raw: [null, 7] },
        },
      },
    ],
    sha256: "b".repeat(64),
  },
  budget: {
    maxFalsePositiveCount: 0,
    maxFalseNegativeCount: 0,
    verdict: "unusable",
    passed: false,
  },
});
test("lint capture accepts observed invalid counts without imposing the historical count", () => {
  const report = makeReport(),
    before = JSON.stringify(report);
  assert.equal(validateReport(report, projects[0], expected), 1);
  assert.equal(JSON.stringify(report), before);
  assert.equal(originalExit({ status: 21, signal: null }), 21);
  assert.equal(originalExit({ status: 0, signal: null }), 0);
  assert.equal(originalExit({ status: null, signal: "SIGTERM" }), 143);
  assert.equal(originalExit({ status: null, signal: null }), 1);
});
test("lint capture refuses mismatched counts, weakened budgets and source or fixture substitution", () => {
  for (const mutate of [
    (r: any) => {
      r.divergence.summary.baselineInvalidRangeCount = 2;
    },
    (r: any) => {
      r.divergence.baselineInvalidRanges = { length: 1 };
    },
    (r: any) => {
      r.budget.verdict = "pass";
    },
    (r: any) => {
      r.budget.passed = true;
    },
    (r: any) => {
      r.budget.maxFalseNegativeCount = 1;
    },
    (r: any) => {
      r.evidence.commitSha = "c".repeat(40);
    },
    (r: any) => {
      r.revision = "c".repeat(40);
    },
    (r: any) => {
      r.divergence.baselineInvalidRanges[0].file = "../App.vue";
    },
  ]) {
    const report = makeReport();
    mutate(report);
    assert.throws(() => validateReport(report, projects[0], expected));
  }
});
test("lint capture refuses an incomplete or duplicated nineteen-project summary", () => {
  const summary = {
    schema: "vize.fixtureLintDivergenceIndex",
    version: 1,
    evidence: { commitSha: expected },
    preset: "ecosystem",
    projectCount: 19,
    budget: { projectCount: 19 },
    projects: projects.map((p) => ({ project: p.id })),
    totals: { baselineInvalidRangeCount: 19 },
  };
  const reports = projects.map(() => ({ actualInvalidCount: 1 }));
  assert.equal(validateSummary(summary, reports, expected), 19);
  assert.throws(() => validateSummary(summary, reports.slice(1), expected));
  summary.projects[0].project = summary.projects[1].project;
  assert.throws(() => validateSummary(summary, reports, expected));
});
const files = [
  { path: "reports/original.json", bytes: Buffer.from(JSON.stringify(makeReport())) },
  { path: "reporter.stderr.log", bytes: Buffer.from("original failure\n") },
];
const header = {
  source: expected,
  runId: "123",
  attempt: "1",
  actualProcessExit: 21,
  actualSignal: null,
  complete: false,
  acceptance: false as const,
  files: files.length,
};
test("lint capture transports complete original bytes and a failed exit through real log frames", () => {
  const log = encodeFrames(header, files)
    .map((line, index) => (index === 2 ? "\uFEFF" : "") + "2026-10-03T09:00:00.123Z " + line)
    .join("\n");
  const decoded = decodeFrames(log, expected);
  assert.deepEqual(decoded.header, header);
  for (const file of files) assert.deepEqual(decoded.files.get(file.path), file.bytes);
});
test("lint capture rejects missing, repeated, corrupt and foreign-source frames", () => {
  const good = encodeFrames(header, files);
  assert.throws(() => decodeFrames(good.slice(0, -1).join("\n"), expected));
  assert.throws(() => decodeFrames([...good, good[0]].join("\n"), expected));
  assert.throws(() => decodeFrames(good.join("\n"), "c".repeat(40)));
  for (const field of ["sha256", "gzipSha256", "bytes", "index"]) {
    const changed = [...good],
      row = JSON.parse(changed[1].slice("VIZE_LINT_RANGE_FILE ".length));
    row[field] = typeof row[field] === "number" ? row[field] + 1 : "c".repeat(64);
    changed[1] = "VIZE_LINT_RANGE_FILE " + JSON.stringify(row);
    assert.throws(() => decodeFrames(changed.join("\n"), expected));
  }
  const swapped = [...good];
  [swapped[1], swapped[2]] = [swapped[2], swapped[1]];
  assert.throws(() => decodeFrames(swapped.join("\n"), expected));
});
test("lint-only dispatch keeps normal suites and preserves enforce failure artifacts", () => {
  const matrix = readFileSync(
    new URL("../../.github/workflows/real-project-matrix.yml", import.meta.url),
    "utf8",
  );
  const workflow = readFileSync(
    new URL("../../.github/workflows/lint-range-capture.yml", import.meta.url),
    "utf8",
  );
  assert.match(matrix, /capture_invalid_lint_ranges:[\s\S]*default: false/u);
  assert.equal(
    (
      matrix.match(
        /github.event_name != 'workflow_dispatch' \|\| !inputs.capture_invalid_lint_ranges/gu,
      ) ?? []
    ).length,
    2,
  );
  assert.match(matrix, /uses: \.\/\.github\/workflows\/lint-range-capture\.yml/u);
  assert.equal(workflow.includes("continue-on-error"), false);
  assert.equal(workflow.includes("pull_request"), false);
  assert.match(workflow, /--locked --profile ci -p vize/u);
  assert.match(workflow, /vp install --frozen-lockfile/u);
  assert.equal((workflow.match(/if: \$\{\{ always\(\) \}\}/gu) ?? []).length, 4);
  assert.ok(workflow.indexOf("reports.ts") < workflow.indexOf("frames.ts"));
  assert.ok(workflow.indexOf("upload-artifact@") < workflow.indexOf("run.ts exit"));
  assert.equal(reporterArgs[reporterArgs.indexOf("--budget-mode") + 1], "enforce");
  assert.equal(
    reporterArgs[reporterArgs.indexOf("--project") + 1],
    projects.map((p) => p.id).join(","),
  );
});
