import assert from "node:assert/strict";
import { test } from "node:test";
import { collectFocusAttempts } from "../differential/focus-history-capture.ts";
import { captureCurrentFocusLane } from "../differential/focus-history-current-capture.ts";
import {
  currentFocusSummary,
  decodeFocusHandled,
  validateFocusCurrentCapture,
} from "../differential/focus-history-current-report.ts";
import { validateFocusCapture } from "../differential/focus-history-report.ts";
import { loadFocusCurrentOracle } from "../differential/focus-history-current-oracle.ts";
import { validateFocusProbe } from "../differential/focus-history.ts";
import { sha256 } from "../differential/harness.mjs";
import {
  attempt,
  fixtures,
  root,
  syntheticCurrentCapture,
} from "./support/focus-history-current.ts";

void test("synthetic current capture v2 grants no equivalence and preserves strict historical v1", () => {
  const current = syntheticCurrentCapture();
  assert.deepEqual(validateFocusCurrentCapture(fixtures, current, current.buildReceipt), {
    plannedOriginalWitnesses: 8,
    legacyCaptured: 8,
    nativeHandled: 8,
    captureFailures: 0,
    acceptedCompleteOracles: 0,
    nativeEquivalent: 0,
    pairedComparisons: 0,
  });
  const historical = loadFocusCurrentOracle(root);
  validateFocusCapture(fixtures, historical, historical.buildReceipt);
  validateFocusProbe(current.buildReceipt);
  validateFocusProbe(historical.buildReceipt);
  assert.throws(() => validateFocusCapture(fixtures, current, current.buildReceipt));
  assert.throws(() => validateFocusCurrentCapture(fixtures, historical, historical.buildReceipt));
  const laundered = structuredClone(current);
  laundered.version = 1;
  assert.throws(() => validateFocusCapture(fixtures, laundered, laundered.buildReceipt));
});

void test("current capture rejects missing custody, incomplete attempts and invented success fields", () => {
  const valid = syntheticCurrentCapture();
  for (const mutate of [
    (r: any) => {
      r.version = 1;
    },
    (r: any) => {
      r.acceptance = "accepted";
    },
    (r: any) => r.rows.pop(),
    (r: any) => {
      r.rows[1] = r.rows[0];
    },
    (r: any) => {
      r.rows[0].inputSha256 = "0".repeat(64);
    },
    (r: any) => {
      r.rows[0].sourceSha256 = "0".repeat(64);
    },
    (r: any) => {
      r.rows[0].inputBase64 += "\n";
    },
    (r: any) => {
      r.rows[0].comparison = { state: "equal" };
    },
    (r: any) => r.rows[0].legacy.attempts.pop(),
    (r: any) => r.rows[0].native.attempts.pop(),
    (r: any) => {
      r.rows[0].native.argv = ["--legacy"];
    },
    (r: any) => {
      r.rows[0].native.state = "refused";
    },
    (r: any) => {
      r.rows[0].native.provenance = { scope: "whole-product" };
    },
    (r: any) => {
      r.rows[0].native.attempts[0].exitStatus = 1;
    },
    (r: any) => {
      r.rows[0].native.attempts[0].signal = "SIGTERM";
    },
    (r: any) => {
      r.rows[0].native.attempts[0].processError = "timeout";
    },
    (r: any) => {
      r.rows[0].native.attempts[0].stdoutSha256 = "0".repeat(64);
    },
    (r: any) => {
      r.rows[0].native.attempts[0].referenceComparison = { state: "equal" };
    },
    (r: any) => {
      r.rows[0].native.attempts[0].stderrBase64 = "eA==";
      r.rows[0].native.attempts[0].stderrSha256 = sha256(Buffer.from("x"));
    },
    (r: any) => {
      r.summary.nativeEquivalent = 8;
    },
    (r: any) => {
      r.summary.acceptedCompleteOracles = 8;
    },
    (r: any) => {
      r.accepted = true;
    },
    (r: any) => {
      r.rows[0].accepted = true;
    },
    (r: any) => {
      r.sourceRevision = "b".repeat(40);
    },
    (r: any) => r.buildReceipt.probes.pop(),
  ]) {
    const changed = structuredClone(valid);
    mutate(changed);
    assert.throws(() => validateFocusCurrentCapture(fixtures, changed, valid.buildReceipt));
  }
});

void test("handled envelope is exact and historical refusal cannot masquerade as full observation", () => {
  const packet = syntheticCurrentCapture();
  const bytes = Buffer.from(packet.rows[0].native.attempts[0].stdoutBase64, "base64");
  const valid = JSON.parse(bytes.toString());
  for (const mutate of [
    (value: any) => {
      value.state = "refused";
    },
    (value: any) => {
      value.context = value.observation;
    },
    (value: any) => {
      value.provenance = { fallback: false };
    },
    (value: any) => {
      value.observation = [];
    },
    (value: any) => {
      value.observation = "Case {\n}\n";
    },
  ]) {
    const changed = structuredClone(valid);
    mutate(changed);
    assert.throws(() => decodeFocusHandled(Buffer.from(`${JSON.stringify(changed)}\n`)));
  }
  assert.throws(() => decodeFocusHandled(bytes.subarray(0, bytes.length - 1)));
  assert.throws(() => decodeFocusHandled(Buffer.from([255, 10])));
  const historical = loadFocusCurrentOracle(root);
  assert.throws(() =>
    decodeFocusHandled(Buffer.from(historical.rows[0].native.attempts[0].stdoutBase64, "base64")),
  );
});

void test("all current failure paths keep both raw attempts and no success or provenance", () => {
  const current = syntheticCurrentCapture();
  const good = Buffer.from(current.rows[0].native.attempts[0].stdoutBase64, "base64");
  const refused = Buffer.from(
    loadFocusCurrentOracle(root).rows[0].native.attempts[0].stdoutBase64,
    "base64",
  );
  const empty = Buffer.alloc(0);
  const result = (stdout: Buffer, extra = {}) => ({
    stdout,
    stderr: empty,
    status: 0,
    signal: null,
    ...extra,
  });
  for (const first of [
    () => {
      throw new Error("actual first invocation threw");
    },
    () => result(Buffer.from("partial\n"), { status: 1, stderr: Buffer.from("failure\n") }),
    () =>
      result(Buffer.from("timeout partial\n"), {
        status: null,
        signal: "SIGTERM",
        error: new Error("timeout"),
      }),
    () => result(Buffer.from("malformed json\n")),
    () => result(Buffer.from([255, 10])),
    () => result(refused),
    () =>
      result(
        Buffer.from(
          `${JSON.stringify({ state: "handled", observation: "Case {\n}\nRuleIdentity {\n}\nObservation {\n}\n" })}\n`,
        ),
      ),
  ]) {
    let calls = 0;
    const invoke: any = () => (++calls === 1 ? first() : result(good));
    const lane = captureCurrentFocusLane("/source/observer", fixtures[0], true, invoke);
    assert.equal(calls, 2);
    assert.equal(lane.state, "failed");
    assert.equal(typeof lane.error, "string");
    assert.deepEqual(Object.keys(lane).sort(), ["argv", "attempts", "error", "state"]);
    assert.equal(lane.attempts.length, 2);
    assert.deepEqual(lane.attempts[1], attempt(good));
    const changed = syntheticCurrentCapture();
    changed.rows[0].native = lane;
    changed.summary = currentFocusSummary(changed.rows);
    assert.equal(
      validateFocusCurrentCapture(fixtures, changed, changed.buildReceipt).nativeHandled,
      7,
    );
  }
});

void test("returned failures preserve original status, signal, error and both complete streams", () => {
  const stdout = Buffer.from("partial original bytes\n");
  const stderr = Buffer.from("original failure\n");
  let calls = 0;
  const invoke: any = () => ({
    stdout,
    stderr,
    status: null,
    signal: "SIGTERM",
    error: new Error(`attempt-${++calls}`),
  });
  const attempts = collectFocusAttempts(
    "/source/observer",
    ["--native"],
    fixtures[0].input,
    invoke,
  );
  assert.equal(calls, 2);
  for (const [index, raw] of attempts.entries())
    assert.deepEqual(raw, {
      ...attempt(stdout),
      stderrBase64: stderr.toString("base64"),
      stderrSha256: sha256(stderr),
      exitStatus: null,
      signal: "SIGTERM",
      processError: `attempt-${index + 1}`,
    });
});

void test("current legacy failures also retain the second whole stream before classification", () => {
  const good = Buffer.from(
    syntheticCurrentCapture().rows[0].legacy.attempts[0].stdoutBase64,
    "base64",
  );
  const empty = Buffer.alloc(0);
  for (const first of [
    () => {
      throw new Error("legacy first invocation failed");
    },
    () => ({ stdout: Buffer.from("partial\n"), stderr: empty, status: 1, signal: null }),
    () => ({ stdout: Buffer.from("malformed legacy\n"), stderr: empty, status: 0, signal: null }),
    () => ({
      stdout: Buffer.from("Case {\n}\nRuleIdentity {\n}\nObservation {\n}\n"),
      stderr: empty,
      status: 0,
      signal: null,
    }),
  ]) {
    let calls = 0;
    const invoke: any = () =>
      ++calls === 1 ? first() : { stdout: good, stderr: empty, status: 0, signal: null };
    const lane = captureCurrentFocusLane("/source/observer", fixtures[0], false, invoke);
    assert.equal(calls, 2);
    assert.equal(lane.state, "failed");
    assert.deepEqual(Object.keys(lane).sort(), ["argv", "attempts", "error", "state"]);
    assert.equal(lane.attempts.length, 2);
    assert.deepEqual(lane.attempts[1], attempt(good));
    const capture = syntheticCurrentCapture();
    capture.rows[0].legacy = lane;
    capture.summary = currentFocusSummary(capture.rows);
    assert.equal(
      validateFocusCurrentCapture(fixtures, capture, capture.buildReceipt).legacyCaptured,
      7,
    );
  }
});
