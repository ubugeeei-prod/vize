import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { sha256 } from "../differential/harness.mjs";
import {
  compareFocusCurrentOutputs,
  FOCUS_CURRENT_CAPTURE_PATH,
  loadFocusCurrentOracle,
  validateFocusCurrentComparison,
} from "../differential/focus-history-current-oracle.ts";
import { focusSummary } from "../differential/focus-history-report.ts";
import { planToolingTests } from "../../tools/support/compat/github/plan-tooling-tests.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const original = fs.readFileSync(path.join(root, FOCUS_CURRENT_CAPTURE_PATH));

// These are data-only validator laws over an actual old packet. Replaying its
// data here never establishes a fresh observer build or native runtime proof.
void test("the immutable historical packet retains its original unreviewed metadata and all attempts", () => {
  const packet = loadFocusCurrentOracle(root);
  assert.equal(packet.acceptance, "unreviewed");
  assert.equal(
    packet.rows.flatMap((row: any) => [...row.legacy.attempts, ...row.native.attempts]).length,
    32,
  );
  assert.equal(packet.summary.acceptedCompleteOracles, 0);
  const comparison = compareFocusCurrentOutputs(root, packet, packet.buildReceipt);
  assert.deepEqual(comparison.summary, {
    reviewedCurrentOutputOracles: 8,
    completeLegacyMatches: 8,
    nativeRefusalMatches: 8,
    nativeRefused: 8,
    currentOutputDrift: 0,
    captureFailures: 0,
    wholeCurrentComparisons: 32,
    historicalCompleteOutputAuthorities: 0,
    nativeHandled: 0,
    nativeEquivalent: 0,
    pairedComparisons: 0,
  });
  validateFocusCurrentComparison(root, comparison, packet.buildReceipt);
  const rewritten = structuredClone(packet);
  rewritten.acceptance = "accepted";
  assert.throws(() => loadFocusCurrentOracle(root, Buffer.from(`${JSON.stringify(rewritten)}\n`)));
  assert.throws(() => loadFocusCurrentOracle(root, original.subarray(0, original.length - 1)));
});

void test("complete matching refuses changed help, coordinates, labels and metadata even when both fresh streams repeat", () => {
  for (const [before, after] of [
    ["Manage focus programmatically", "Change focus programmatically"],
    ["start: 19,", "start: 20,"],
    ["labels: [],", 'labels: ["invented"],'],
    ["constructor_default_help: Full,", "constructor_default_help: None,"],
  ]) {
    const packet = loadFocusCurrentOracle(root);
    for (const attempt of packet.rows[0].legacy.attempts) {
      const body = Buffer.from(attempt.stdoutBase64, "base64").toString("utf8");
      assert(body.includes(before));
      const changed = Buffer.from(body.replaceAll(before, after));
      attempt.stdoutBase64 = changed.toString("base64");
      attempt.stdoutSha256 = sha256(changed);
    }
    const comparison = compareFocusCurrentOutputs(root, packet, packet.buildReceipt);
    assert.equal(comparison.summary.currentOutputDrift, 1);
    assert.equal(comparison.summary.completeLegacyMatches, 7);
    validateFocusCurrentComparison(root, comparison, packet.buildReceipt);
  }
});

function replaceAttempts(lane: any, from: string, to: string) {
  for (const attempt of lane.attempts) {
    const originalBytes = Buffer.from(attempt.stdoutBase64, "base64");
    const body = originalBytes.toString("utf8");
    assert(body.includes(from));
    const changed = Buffer.from(body.replaceAll(from, to));
    assert(!changed.equals(originalBytes));
    attempt.stdoutBase64 = changed.toString("base64");
    attempt.stdoutSha256 = sha256(changed);
  }
}

void test("complete metadata, fixes, severity, requery and empty vectors cannot drift behind repeated streams", () => {
  for (const [from, to] of [
    ['filename: "test.vue"', 'filename: "changed.vue"'],
    ["category: Accessibility", "category: Essential"],
    ["severity: Warning", "severity: Error"],
    ["fix: None", 'fix: Some("invented")'],
    ["unchanged_requery: Some", "unchanged_requery: None"],
  ]) {
    const packet = loadFocusCurrentOracle(root);
    replaceAttempts(packet.rows[0].legacy, from, to);
    const comparison = compareFocusCurrentOutputs(root, packet, packet.buildReceipt);
    assert.equal(comparison.rows[0].legacy.state, "current-output-drift");
    assert.equal(comparison.summary.completeLegacyMatches, 7);
  }
  const packet = loadFocusCurrentOracle(root);
  replaceAttempts(packet.rows[2].legacy, "diagnostics: []", 'diagnostics: ["invented warning"]');
  const comparison = compareFocusCurrentOutputs(root, packet, packet.buildReceipt);
  assert.equal(comparison.summary.currentOutputDrift, 1);
  assert.equal(comparison.summary.nativeEquivalent, 0);
});

void test("native refusal context and every stdout byte are compared to the single immutable current packet", () => {
  for (const [from, to] of [
    ['filename: \\"test.vue\\"', 'filename: \\"changed.vue\\"'],
    ["category: Accessibility", "category: Essential"],
    ["constructor_default_help: Full", "constructor_default_help: Short"],
  ]) {
    const packet = loadFocusCurrentOracle(root);
    replaceAttempts(packet.rows[0].native, from, to);
    const comparison = compareFocusCurrentOutputs(root, packet, packet.buildReceipt);
    assert.equal(comparison.rows[0].native.state, "current-output-drift");
    assert.equal(comparison.summary.nativeRefusalMatches, 7);
    assert.equal(comparison.summary.nativeRefused, 8);
    assert.equal(comparison.summary.nativeHandled, 0);
    assert.deepEqual(comparison.rows[0].comparison, {
      state: "not-compared",
      reason: "unprovided-rule",
    });
  }
});

void test("a failed first process retains both full streams and their differences before aggregate rejection", () => {
  for (const name of ["legacy", "native"]) {
    const packet = loadFocusCurrentOracle(root);
    const lane = packet.rows[0][name];
    lane.state = "failed";
    lane.error = "actual first process failed";
    const first = lane.attempts[0];
    const partial = Buffer.from("partial\n");
    first.stdoutBase64 = partial.toString("base64");
    first.stdoutSha256 = sha256(partial);
    first.exitStatus = 1;
    packet.summary = focusSummary(packet.rows);
    const comparison = compareFocusCurrentOutputs(root, packet, packet.buildReceipt);
    assert.equal(comparison.rows[0][name].state, "failed");
    assert.deepEqual(comparison.capture, packet);
    assert.equal(comparison.rows[0][name].wholeComparisons[0].state, "different");
    assert.equal(comparison.rows[0][name].wholeComparisons[1].state, "equal");
    assert.equal(comparison.summary.captureFailures, 1);
    assert.equal(comparison.summary.wholeCurrentComparisons, 32);
    validateFocusCurrentComparison(root, comparison, packet.buildReceipt);
  }
});

void test("complete comparison validation rejects forged clean rows, source custody, attempts and native or historical credit", () => {
  const packet = loadFocusCurrentOracle(root);
  const comparison = compareFocusCurrentOutputs(root, packet, packet.buildReceipt);
  for (const mutate of [
    (value: any) => {
      value.version = 1;
    },
    (value: any) => {
      value.summary.nativeHandled = 8;
    },
    (value: any) => {
      value.summary.nativeEquivalent = 8;
    },
    (value: any) => {
      value.summary.pairedComparisons = 8;
    },
    (value: any) => {
      value.summary.historicalCompleteOutputAuthorities = 8;
    },
    (value: any) => {
      value.rows[0].comparison = { state: "equal" };
    },
    (value: any) => {
      value.rows[0].native.provenance = { scope: "whole-product" };
    },
    (value: any) => {
      value.rows[0].legacy.wholeComparisons.pop();
    },
    (value: any) => {
      value.capture.rows[0].native.attempts.pop();
    },
    (value: any) => {
      value.capture.acceptance = "accepted";
    },
    (value: any) => {
      value.capture.buildReceipt.source.sourceRevision = "0".repeat(40);
    },
    (value: any) => {
      value.qualification.authority = "historical-full-output";
    },
  ]) {
    const changed = structuredClone(comparison);
    mutate(changed);
    assert.throws(() => validateFocusCurrentComparison(root, changed, packet.buildReceipt));
  }
});

void test("canonical report controls keep pure T0, full T1, stale-success deletion and one genuine build/capture", () => {
  const execution = "tests/tooling/focus-history-capture-execution.test.ts";
  const pure = "tests/tooling/focus-history-current-oracle.test.ts";
  for (const input of [
    FOCUS_CURRENT_CAPTURE_PATH,
    "tests/differential/focus-history-current-oracle.ts",
    pure,
  ]) {
    const pr = planToolingTests([input], { cwd: root }).tests;
    const full = planToolingTests([input], { tier: "merge", cwd: root }).tests;
    assert(!pr.includes(execution));
    assert(pr.includes(pure));
    assert(full.includes(execution) && full.includes(pure));
  }
  const source = fs.readFileSync(path.join(root, execution), "utf8");
  const staleDelete = source.indexOf("fs.rmSync(comparisonPath");
  const build = source.indexOf("buildProductObserver({");
  const raw = source.indexOf('"unaccepted-capture.json"');
  const write = source.indexOf("fs.writeFileSync(comparisonPath");
  const validate = source.indexOf("validateFocusCurrentComparison(root");
  const aggregate = source.indexOf("assert.deepEqual(");
  assert(
    staleDelete >= 0 &&
      staleDelete < build &&
      build < raw &&
      raw < write &&
      write < validate &&
      validate < aggregate,
  );
  assert.equal(source.split("buildProductObserver({").length - 1, 1);
  assert.equal(source.split("runFocusCapture({").length - 1, 1);
});

void test("a failed or missing capture cannot inherit an old successful current-output match", () => {
  for (const mutate of [
    (packet: any) => packet.rows.pop(),
    (packet: any) => {
      packet.rows[0].legacy.attempts[0].exitStatus = 1;
    },
    (packet: any) => {
      packet.rows[0].native.attempts[0].signal = "SIGTERM";
    },
    (packet: any) => {
      packet.summary.nativeEquivalent = 8;
    },
  ]) {
    const packet = loadFocusCurrentOracle(root);
    mutate(packet);
    assert.throws(() => compareFocusCurrentOutputs(root, packet, packet.buildReceipt));
  }
});
