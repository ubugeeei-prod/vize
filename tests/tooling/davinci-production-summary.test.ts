import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test, type TestContext } from "node:test";
import { fileURLToPath } from "node:url";

const summaryScript = fileURLToPath(
  new URL("../../tools/benchmarks/scripts/davinci-production-summary.mjs", import.meta.url),
);
const shapes = ["dom_inline", "dom_module", "ssr", "vapor"];

function observations(shape: string) {
  const observation = (overrides: Record<string, unknown> = {}) => ({
    has_template: true,
    compiled: true,
    backend: shape,
    selected_lane: "accepted",
    cohort: "accepted",
    code_equal: true,
    messages_equal: true,
    ...overrides,
  });
  return [
    observation(),
    observation({ cohort: "diagnostic" }),
    observation({ selected_lane: "legacy.emit_refused", cohort: "fallback" }),
    shape.startsWith("dom_")
      ? observation({ backend: "vapor", cohort: "routed_vapor" })
      : observation({ selected_lane: "unrecorded", cohort: "fallback" }),
    observation({ has_template: false, selected_lane: "unrecorded", cohort: "no_template" }),
    observation({ compiled: false, selected_lane: "unrecorded", cohort: "diagnostic" }),
  ];
}

function report() {
  return {
    schema_version: 1,
    head_sha: "a".repeat(40),
    manifest_sha256: "b".repeat(64),
    files: 6,
    profile: "release",
    allocator: "system",
    features: [],
    options: {},
    window: "whole-sfc",
    sampling: "paired",
    shapes: shapes.map((shape) => ({
      shape,
      observations: observations(shape),
      cohorts: {
        all: {
          files: 6,
          passes_per_sample: 5,
          selected_batch_ns: Array<number>(9).fill(10),
          retained_batch_ns: Array<number>(9).fill(20),
          selected_median_batch_ns: 10,
          retained_median_batch_ns: 20,
        },
      },
    })),
  };
}

function summarize(t: TestContext, mutate?: (reports: ReturnType<typeof report>[]) => void) {
  const root = mkdtempSync(join(tmpdir(), "vize-production-summary-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const reports = [report(), report(), report()];
  mutate?.(reports);
  for (const [index, value] of reports.entries()) {
    const directory = join(root, `davinci-production-perf-${index + 1}`);
    mkdirSync(directory);
    writeFileSync(join(directory, "samples.json"), JSON.stringify(value));
  }
  const output = join(root, "summary.md");
  const result = spawnSync(process.execPath, [summaryScript, root, output], {
    encoding: "utf8",
    env: { ...process.env, GITHUB_STEP_SUMMARY: "" },
  });
  return { result, output };
}

test("accepted emitter selections never become native-only acceptance", (t) => {
  const { result, output } = summarize(t);
  assert.equal(result.status, 0, result.stderr);
  const summary = readFileSync(output, "utf8");
  assert.match(summary, /Native-only acceptance is not measured by these reports\./u);
  assert.match(summary, /Croquis-backed binding and reactivity facts/u);
  assert.match(summary, /Emitter selected for requested backend/u);
  assert.doesNotMatch(summary, /Native for requested backend/u);
  // Diagnosed selections remain emitter observations; errors and absent
  // templates never enter the compiled-template denominator.
  assert.ok(summary.includes("| dom_inline | 4 | 2 | 1 | 1 | 0 | 0 |"));
  assert.ok(summary.includes("| ssr | 4 | 2 | 0 | 2 | 0 | 0 |"));
  assert.ok(summary.includes("| dom_inline | all | 6 | 0.5000 |"));
});

for (const invalid of ["duplicate", "missing", "unknown"] as const) {
  test(`shipping-shape coverage rejects ${invalid} entries without publishing a report`, (t) => {
    const { result, output } = summarize(t, (reports) => {
      for (const value of reports) {
        if (invalid === "missing") value.shapes.pop();
        else value.shapes[3].shape = invalid === "duplicate" ? "ssr" : "unknown";
      }
    });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Expected each shipping shape exactly once/u);
    assert.equal(existsSync(output), false);
  });
}

test("each independent runner must cover all four shapes", (t) => {
  const { result, output } = summarize(t, (reports) => {
    reports[1].shapes[3].shape = "ssr";
  });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /Expected each shipping shape exactly once/u);
  assert.equal(existsSync(output), false);
});

test("different admission observations cannot publish an aggregate", (t) => {
  const { result, output } = summarize(t, (reports) => {
    reports[2].shapes[0].observations[0].selected_lane = "legacy.croquis";
  });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /Nondeterministic output\/admission observations/u);
  assert.equal(existsSync(output), false);
});

test("different heads cannot publish an aggregate", (t) => {
  const { result, output } = summarize(t, (reports) => {
    reports[1].head_sha = "c".repeat(40);
  });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /Reports have different input\/build identities/u);
  assert.equal(existsSync(output), false);
});
