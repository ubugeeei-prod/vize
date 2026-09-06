// Davinci P0-13 — seeded-defect + suppression-telemetry pilot oracles.
//
// Exercises both FP/FN pilot tools over the COMMITTED miniature fixture set
// (tests/_fixtures/davinci-fpfn) so CI proves the identity assertion without
// corpus hydration; the corpus-shard run stays a local/nightly concern
// (davinci-road/plan/phase-0.md, P0-13).
//
// The committed expected/assert-report.json pins the MEASURED current
// toolchain behavior: class-(a) recall is 0/3 because vue/no-undefined-refs
// is registered by no preset and no opt-in path, so `vize lint` cannot fire
// it (davinci-road/plan/ledger-fn.md, FN-1). The day that rule gains a
// consumer this test fails loudly — refresh expected/assert-report.json AND
// flip the FN-1 ledger entry in the same change.

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixtures = path.join(root, "tests/_fixtures/davinci-fpfn");
const expectedDir = path.join(fixtures, "expected");
const seedTool = path.join(root, "tools/commands/davinci/seed-defects.rs");
const suppressionTool = path.join(root, "tools/commands/davinci/suppression-telemetry.rs");

function runTool(tool: string, args: string[]) {
  const result = spawnSync("rust-script", [tool, ...args], { cwd: root, encoding: "utf8" });
  if (result.error) throw result.error;
  return result;
}

function readJson(filePath: string): unknown {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function tempDir(label: string): string {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), `davinci-fpfn-${label}-`));
  process.on("exit", () => fs.rmSync(dir, { recursive: true, force: true }));
  return dir;
}

// One seeded tree is shared by the assertion tests below; seeding is
// deterministic (verified first), so reuse does not order-couple the tests.
const seedOut = tempDir("seed");
const seedRun = runTool(seedTool, ["--fixtures", fixtures, "--out", seedOut]);

test("seed-defects seeds the committed fixture set and prints scope proof", () => {
  assert.equal(seedRun.status, 0, `${seedRun.stdout}\n${seedRun.stderr}`);
  const lines = seedRun.stdout.trim().split("\n");
  assert.equal(
    lines[lines.length - 1],
    "scope-proof: files-scanned=4 class-a-eligible=3 class-a-injections=3 class-b-injections=4",
  );
});

test("the emitted manifest byte-matches the committed expectation", () => {
  const manifest = readJson(path.join(seedOut, "manifest.json"));
  assert.deepStrictEqual(manifest, readJson(path.join(expectedDir, "manifest.json")));
});

test("seeding is deterministic: a second run is byte-identical", () => {
  const again = tempDir("seed-again");
  const rerun = runTool(seedTool, ["--fixtures", fixtures, "--out", again]);
  assert.equal(rerun.status, 0, `${rerun.stdout}\n${rerun.stderr}`);
  assert.equal(
    fs.readFileSync(path.join(again, "manifest.json"), "utf8"),
    fs.readFileSync(path.join(seedOut, "manifest.json"), "utf8"),
  );
  const manifest = readJson(path.join(seedOut, "manifest.json")) as {
    files: { path: string }[];
  };
  for (const file of manifest.files) {
    assert.equal(
      fs.readFileSync(path.join(again, "seeded", file.path), "utf8"),
      fs.readFileSync(path.join(seedOut, "seeded", file.path), "utf8"),
      `seeded ${file.path} differs between runs`,
    );
  }
});

test("identity assertion measures current recall exactly (0/3, each miss listed)", () => {
  const reportPath = path.join(seedOut, "assert-report.json");
  const result = runTool(seedTool, ["--assert", "--out", seedOut, "--report", reportPath]);
  assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
  assert.deepStrictEqual(
    readJson(reportPath),
    readJson(path.join(expectedDir, "assert-report.json")),
  );
});

// --- assertion-mechanism self-tests (synthetic lint JSON hooks) -------------

type LintMessage = {
  ruleId: string;
  ruleDocsPath: string;
  severity: number;
  message: string;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
};

type ManifestInjection = {
  class: string;
  path: string;
  expectedRule: string | null;
  expected: { line: number; column: number; endLine: number; endColumn: number };
};

function syntheticMessage(
  ruleId: string,
  span: { line: number; column: number; endLine: number; endColumn: number },
): LintMessage {
  return {
    ruleId,
    ruleDocsPath: "docs/content/rules/vue.md",
    severity: 1,
    message: `[vize:${ruleId}] synthetic`,
    line: span.line,
    column: span.column,
    endLine: span.endLine,
    endColumn: span.endColumn,
  };
}

// The one real pristine-tree diagnostic of the fixture set, in original
// coordinates: BaselineDrift.vue keeps an unsuppressed multi-space run so
// the baseline-shift machinery is exercised (class-(b) inserts one script
// line above it). If default-preset lint output over the fixtures ever
// changes, the real-run test above fails first and points at the drift.
const BASELINE_MULTI_SPACE = { line: 6, column: 5, endLine: 6, endColumn: 7 };
const SHIFTED_MULTI_SPACE = { line: 7, column: 5, endLine: 7, endColumn: 7 };

function syntheticTrees() {
  const manifest = readJson(path.join(expectedDir, "manifest.json")) as {
    files: { path: string }[];
    injections: ManifestInjection[];
  };
  const baseline = manifest.files.map((file) => ({
    file: file.path,
    messages:
      file.path === "BaselineDrift.vue"
        ? [syntheticMessage("vue/no-multi-spaces", BASELINE_MULTI_SPACE)]
        : [],
    errorCount: 0,
    warningCount: file.path === "BaselineDrift.vue" ? 1 : 0,
  }));
  const seeded = manifest.files.map((file) => {
    const messages: LintMessage[] = [];
    if (file.path === "BaselineDrift.vue") {
      messages.push(syntheticMessage("vue/no-multi-spaces", SHIFTED_MULTI_SPACE));
    }
    for (const injection of manifest.injections) {
      if (injection.class === "undefined-template-ref" && injection.path === file.path) {
        assert.equal(injection.expectedRule, "vue/no-undefined-refs");
        messages.push(syntheticMessage(injection.expectedRule, injection.expected));
      }
    }
    return { file: file.path, messages, errorCount: 0, warningCount: messages.length };
  });
  return { baseline, seeded };
}

function runSyntheticAssert(label: string, mutate: (seeded: unknown[]) => void) {
  const { baseline, seeded } = syntheticTrees();
  mutate(seeded);
  const baselinePath = path.join(seedOut, `synthetic-${label}-baseline.json`);
  const seededPath = path.join(seedOut, `synthetic-${label}-seeded.json`);
  fs.writeFileSync(baselinePath, JSON.stringify(baseline, null, 2));
  fs.writeFileSync(seededPath, JSON.stringify(seeded, null, 2));
  const reportPath = path.join(seedOut, `synthetic-${label}-report.json`);
  const result = runTool(seedTool, [
    "--assert",
    "--out",
    seedOut,
    "--baseline-lint-json",
    baselinePath,
    "--seeded-lint-json",
    seededPath,
    "--report",
    reportPath,
  ]);
  return { result, report: readJson(reportPath) };
}

test("assertion mechanism: the exact expected diagnostic set passes", () => {
  const { result, report } = runSyntheticAssert("pass", () => {});
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  const summary = report as {
    verdict: string;
    classA: { expected: number; detected: number; misses: unknown[] };
    baselineShift: { mapped: number; misses: unknown[]; unmappable: unknown[] };
    unexpected: unknown[];
  };
  assert.equal(summary.verdict, "pass");
  assert.deepStrictEqual(
    {
      expected: summary.classA.expected,
      detected: summary.classA.detected,
      misses: summary.classA.misses,
    },
    { expected: 3, detected: 3, misses: [] },
  );
  assert.deepStrictEqual(summary.baselineShift, { mapped: 1, misses: [], unmappable: [] });
  assert.deepStrictEqual(summary.unexpected, []);
});

test("assertion mechanism: identity, not count — a moved diagnostic fails", () => {
  const { result, report } = runSyntheticAssert("moved", (seeded) => {
    // Same diagnostic COUNT, wrong location: shift the BaselineDrift
    // class-(a) diagnostic one column right. Count-only matching would
    // pass this; the identity oracle must not.
    const drift = (seeded as { file: string; messages: LintMessage[] }[]).find(
      (entry) => entry.file === "BaselineDrift.vue",
    );
    assert.ok(drift);
    const target = drift.messages.find((message) => message.ruleId === "vue/no-undefined-refs");
    assert.ok(target);
    target.column += 1;
  });
  assert.equal(result.status, 1, `${result.stdout}\n${result.stderr}`);
  const summary = report as {
    verdict: string;
    classA: { misses: unknown[] };
    unexpected: unknown[];
  };
  assert.equal(summary.verdict, "fail");
  assert.deepStrictEqual(summary.classA.misses, [
    {
      path: "BaselineDrift.vue",
      ruleId: "vue/no-undefined-refs",
      severity: 1,
      line: 7,
      column: 24,
      endLine: 7,
      endColumn: 31,
      identifier: "message",
    },
  ]);
  assert.deepStrictEqual(summary.unexpected, [
    {
      path: "BaselineDrift.vue",
      ruleId: "vue/no-undefined-refs",
      severity: 1,
      line: 7,
      column: 25,
      endLine: 7,
      endColumn: 31,
    },
  ]);
});

test("suppression telemetry reports the mapped FP candidate and the unmapped name", () => {
  const out = tempDir("suppression");
  const reportPath = path.join(out, "report.json");
  const result = runTool(suppressionTool, [
    "--fixtures",
    fixtures,
    "--out",
    out,
    "--report",
    reportPath,
  ]);
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  assert.deepStrictEqual(
    readJson(reportPath),
    readJson(path.join(expectedDir, "suppression-report.json")),
  );
  const lines = result.stdout.trim().split("\n");
  assert.equal(
    lines[1],
    "scope-proof: files-scanned=4 suppression-comments=2 named=2 bare=0 " +
      "rules-mapped=123 mapped-seen=1 unmapped-seen=1 fp-candidates=1",
  );
});
