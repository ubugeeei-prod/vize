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
} from "../differential/focus-history-current-oracle.ts";

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
    nativeRefused: 8,
    nativeHandled: 0,
    nativeEquivalent: 0,
    pairedComparisons: 0,
  });
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
    assert.throws(() => compareFocusCurrentOutputs(root, packet, packet.buildReceipt));
  }
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
