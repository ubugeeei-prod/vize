import assert from "node:assert/strict";
import { readFileSync, writeFileSync, mkdtempSync, mkdirSync, rmSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";
import { verifyTrackedSource } from "../../tools/support/ci/lint-range-capture/preflight.ts";
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

test("lint capture retains actual dirty gitlink evidence without allowing the producer", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-lint-capture-drift-"));
  const invoke = (cwd: string, ...args: string[]) =>
    execFileSync("git", ["-C", cwd, ...args], { stdio: ["ignore", "pipe", "pipe"] });
  const commit = (cwd: string) => {
    invoke(cwd, "add", ".");
    invoke(
      cwd,
      "-c",
      "user.name=Capture Law",
      "-c",
      "user.email=capture@example.invalid",
      "commit",
      "-qm",
      "fixture",
    );
  };
  try {
    const project = projects[3];
    const cwd = join(root, project.fixturePath);
    mkdirSync(cwd, { recursive: true });
    invoke(cwd, "init", "-q");
    writeFileSync(join(cwd, "App.vue"), "original fixture\n");
    commit(cwd);
    const fixtureHead = invoke(cwd, "rev-parse", "HEAD").toString().trim();
    invoke(root, "init", "-q");
    writeFileSync(join(root, "control.txt"), "original control\n");
    commit(root);
    const head = invoke(root, "rev-parse", "HEAD").toString().trim();
    verifyTrackedSource(root, head);
    writeFileSync(join(cwd, "App.vue"), "observed changed fixture\n");
    assert.throws(() => verifyTrackedSource(root, head), /Tracked source has edits/u);
    const path = join(root, "eslint-invalid-range-capture/source-drift.json");
    const originalRecord = readFileSync(path);
    const record = JSON.parse(originalRecord.toString());
    assert.equal(record.expectedSource, head);
    assert.equal(record.actualSource, head);
    assert.equal(record.trackedPaths, project.fixturePath + "\n");
    assert.match(record.repositoryStatus, /M tests\/_fixtures\/_git\/shadcn-vue/u);
    assert.equal(record.fixtures.length, 1);
    assert.equal(record.fixtures[0].expectedRevision, project.revision);
    assert.equal(record.fixtures[0].actualRevision, fixtureHead);
    assert.match(record.fixtures[0].status, /M App\.vue/u);
    assert.match(record.fixtures[0].diff, /-original fixture/u);
    assert.match(record.fixtures[0].diff, /\+observed changed fixture/u);
    assert.equal(record.producerExecuted, false);
    assert.equal(record.acceptance, false);
    writeFileSync(join(cwd, "App.vue"), "original fixture\n");
    verifyTrackedSource(root, head);
    writeFileSync(join(root, "control.txt"), "later root change\n");
    assert.throws(() => verifyTrackedSource(root, head), /Tracked source has edits/u);
    assert.deepEqual(readFileSync(path), originalRecord);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("lint capture retains a real Git diff above one MiB through unchanged full frames", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-lint-capture-large-drift-"));
  const invoke = (...args: string[]) =>
    execFileSync("git", ["-C", root, ...args], { stdio: ["ignore", "pipe", "pipe"] });
  const sha = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
  try {
    invoke("init", "-q");
    writeFileSync(join(root, "fixture.txt"), "original\n");
    invoke("add", ".");
    invoke(
      "-c",
      "user.name=Capture Law",
      "-c",
      "user.email=capture@example.invalid",
      "commit",
      "-qm",
      "fixture",
    );
    const head = invoke("rev-parse", "HEAD").toString().trim();
    writeFileSync(
      join(root, "fixture.txt"),
      "observed changed fixture line abcdefghijklmnopqrstuvwxyz\n".repeat(24000),
    );
    const expectedDiff = execFileSync(
      "git",
      ["-C", root, "diff", "--no-ext-diff", "--submodule=diff", "HEAD"],
      { maxBuffer: 4 * 1024 * 1024 },
    );
    assert.ok(expectedDiff.length > 1024 * 1024);
    assert.throws(() => verifyTrackedSource(root, head), /Tracked source has edits/u);
    const out = join(root, "eslint-invalid-range-capture");
    const recordBytes = readFileSync(join(out, "source-drift.json"));
    const record = JSON.parse(recordBytes.toString());
    assert.equal(typeof record.repositoryDiff, "object");
    assert.equal(record.repositoryDiff.bytes, expectedDiff.length);
    assert.equal(record.repositoryDiff.sha256, sha(expectedDiff));
    assert.deepEqual(readFileSync(join(out, record.repositoryDiff.path)), expectedDiff);
    for (const row of record.gitOutputs)
      for (const file of [row.stdout, row.stderr]) {
        const bytes = readFileSync(join(out, file.path));
        assert.equal(bytes.length, file.bytes);
        assert.equal(sha(bytes), file.sha256);
      }
    const frames = execFileSync(process.execPath, [
      fileURLToPath(
        new URL("../../tools/support/ci/lint-range-capture/frames.ts", import.meta.url),
      ),
      root,
      head,
    ]).toString();
    const decoded = decodeFrames(frames, head);
    assert.deepEqual(decoded.files.get("source-drift.json"), recordBytes);
    assert.deepEqual(decoded.files.get(record.repositoryDiff.path), expectedDiff);
    assert.equal(decoded.header.complete, false);
    assert.equal(decoded.header.actualProcessExit, null);
    assert.equal(decoded.header.acceptance, false);
    writeFileSync(join(root, "fixture.txt"), "later change\n");
    assert.throws(() => verifyTrackedSource(root, head), /Tracked source has edits/u);
    assert.deepEqual(readFileSync(join(out, "source-drift.json")), recordBytes);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
