import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { loadLinterManifest, validateLinterReport } from "../differential/linter-api.ts";
import { sha256 } from "../differential/harness.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const manifestPath = path.join(repoRoot, "tests/_fixtures/differential/linter/manifest.json");

void test("component filename history retains the eight original input and option witnesses", () => {
  const file = path.join(
    repoRoot,
    "crates/vize_patina/tests/fixtures/component-name-history/cases.json",
  );
  const bytes = fs.readFileSync(file);
  assert.equal(sha256(bytes), "867a29d7c0a92c17d59fcaa6063be1e3649fb92375024fcd2932a57f27fe062c");
  const cases = JSON.parse(bytes.toString());
  assert.equal(new Set(cases.map((fixture: any) => fixture.id)).size, 8);
  assert.deepEqual(
    cases.map((fixture: any) => [fixture.filename, fixture.diagnostics]),
    [
      ["my-component.vue", 0],
      ["src/components/job-board.vue", 0],
      ["grid-2-col.vue", 0],
      ["my-Component.vue", 1],
      ["-my-component.vue", 1],
      ["my-component-.vue", 1],
      ["my--component.vue", 1],
      ["page.block.vue", 1],
    ],
  );
  for (const fixture of cases) {
    assert.equal(fixture.history, "eaafa5a1f67883277407fcf1c5f3f2b2101ef2ed");
    assert.equal(fixture.source, "<div>Content</div>");
    assert.equal(fixture.entry, "template");
    assert.equal(fixture.rule, "vue/component-definition-name-casing");
    assert.equal(fixture.vue_version, null);
    assert.equal(fixture.vapor, null);
    assert.equal(fixture.fixes, 0);
  }
});

void test("linter registry pins every authored input and complete oracle, including real fixes and reports", () => {
  const loaded = loadLinterManifest(manifestPath, repoRoot);
  assert.equal(loaded.cases.length, 36);
  const cohorts: Record<string, number> = {};
  for (const fixture of loaded.cases) {
    const cohort = fixture.id.split("/")[1];
    cohorts[cohort] = (cohorts[cohort] ?? 0) + 1;
    assert(fixture.expected.length > 0);
    assert(
      fixture.expected.includes(Buffer.from("Case {")) ||
        fixture.expected.includes(Buffer.from("(\n    Case {")),
    );
  }
  assert.deepEqual(cohorts, { "current-api": 13, "next-tick": 10, report: 4, "static-class": 9 });
  const fixed = loaded.cases.find((fixture: any) =>
    fixture.id.endsWith("static-class-utf8-fix-corrected"),
  );
  assert(fixed.expected.includes(Buffer.from("applications: [")));
  assert(fixed.expected.includes(Buffer.from("edits: [")));
  const report = loaded.cases.find(
    (fixture: any) => fixture.id === "linter/report/unicode-crlf-columns",
  );
  assert(
    report.expected.includes(Buffer.from("json:")) &&
      report.expected.includes(Buffer.from("text:")),
  );
});

void test("linter manifest rejects omitted/duplicate inputs, unregistered API and oracle inventory drift", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-linter-manifest-contract-"));
  const file = path.join(root, "manifest.json");
  const baseline = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  try {
    for (const mutate of [
      (manifest: any) => manifest.cases.pop(),
      (manifest: any) => manifest.cases.push(manifest.cases[0]),
      (manifest: any) => {
        manifest.cohorts[0].api = "--private-parser";
      },
      (manifest: any) => {
        manifest.cohorts[0].cases.sha256 = "0".repeat(64);
      },
      (manifest: any) => {
        manifest.oracles.sha256 = "0".repeat(64);
      },
      (manifest: any) => {
        manifest.adapterOptions.fixes = "filtered";
      },
    ]) {
      const manifest = structuredClone(baseline);
      mutate(manifest);
      fs.writeFileSync(file, JSON.stringify(manifest));
      assert.throws(() => loadLinterManifest(file, repoRoot));
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("linter result contract rejects omitted rows, incomplete retries and invented native acceptance", () => {
  const loaded = loadLinterManifest(manifestPath, repoRoot);
  // Synthetic rows validate rejection laws only; no binary is run and no actual
  // fixture/native acceptance is claimed by this contract test.
  const receipt = { source: { sourceRevision: "a".repeat(40) } };
  const rows = loaded.cases.map((fixture: any) => {
    const attempt = {
      stdoutBase64: fixture.expected.toString("base64"),
      stdoutSha256: sha256(fixture.expected),
      stderrBase64: "",
      stderrSha256: sha256(Buffer.alloc(0)),
      exitStatus: 0,
      signal: null,
      processError: null,
      referenceComparison: { state: "equal" },
    };
    return {
      id: fixture.id,
      target: "lint",
      legacy: {
        state: "completed",
        verdict: "matched-reference",
        argv: fixture.argv,
        inputSha256: sha256(fixture.input),
        attempts: [attempt, { ...attempt }],
      },
      native: { state: "unsupported", reason: "whole-product native linter path unavailable" },
      comparison: { state: "not-compared" },
    };
  });
  const baseline = {
    schema: "vize.differential.result",
    version: 1,
    product: "linter",
    sourceRevision: receipt.source.sourceRevision,
    manifestSha256: loaded.manifestSha256,
    buildReceipt: receipt,
    rows,
    summary: {
      plannedCases: 36,
      legacyMatches: 36,
      legacyFailures: 0,
      baselineDrift: 0,
      nativeUnsupported: 36,
      nativeHandled: 0,
      nativeEquivalent: 0,
      pairedComparisons: 0,
    },
  };
  validateLinterReport(loaded, baseline, receipt);
  for (const mutate of [
    (report: any) => report.rows.pop(),
    (report: any) => {
      report.rows[1] = report.rows[0];
    },
    (report: any) => report.rows[0].legacy.attempts.pop(),
    (report: any) => {
      report.rows[0].legacy.attempts[0].exitStatus = 1;
    },
    (report: any) => {
      report.rows[0].legacy.attempts[0].stdoutBase64 = "";
    },
    (report: any) => {
      report.rows[0].native.state = "completed";
    },
    (report: any) => {
      report.rows[0].comparison.state = "equal";
    },
    (report: any) => {
      report.summary.nativeHandled = 1;
    },
  ]) {
    const report = structuredClone(baseline);
    mutate(report);
    assert.throws(() => validateLinterReport(loaded, report, receipt));
  }
});
