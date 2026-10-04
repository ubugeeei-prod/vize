import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { sha256 } from "../differential/harness.mjs";
import { collectFocusAttempts } from "../differential/focus-history-capture.ts";
import { focusSummary, validateFocusCapture } from "../differential/focus-history-report.ts";
import { FOCUS_CONTRACT, loadFocusCases } from "../differential/focus-history.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixtures = loadFocusCases(root);

function probe(value = FOCUS_CONTRACT) {
  const bytes = Buffer.from(`${JSON.stringify(value)}\n`);
  return {
    argv: ["--contract"],
    exitStatus: 0,
    stdoutBase64: bytes.toString("base64"),
    sha256: sha256(bytes),
  };
}
const receipt = { source: { sourceRevision: "a".repeat(40) }, probes: [probe()] };
function attempt(bytes: Buffer) {
  return {
    stdoutBase64: bytes.toString("base64"),
    stdoutSha256: sha256(bytes),
    stderrBase64: "",
    stderrSha256: sha256(Buffer.alloc(0)),
    exitStatus: 0,
    signal: null,
    processError: null,
  };
}

// Deliberately synthetic structural packets test validator laws only. They are
// neither actual LintResult observations nor complete expected-output goldens.
function syntheticReport() {
  const context = "Case {\n    synthetic: true,\n}\nRuleIdentity {\n    synthetic: true,\n}\n";
  const legacy = Buffer.from(`${context}Observation {\n    synthetic: true,\n}\n`);
  const rows = fixtures.map((fixture: any) => {
    const native = Buffer.from(
      `${JSON.stringify({ state: "refused", context, refusal: { kind: "UnprovidedRule", detail: `UnprovidedRule { rule: "${fixture.authored.rule}" }`, unprovided_rule: fixture.authored.rule } })}\n`,
    );
    return {
      id: fixture.id,
      inputBase64: fixture.input.toString("base64"),
      inputSha256: sha256(fixture.input),
      sourceSha256: fixture.authored.source_sha256,
      legacy: {
        state: "captured",
        argv: ["--legacy"],
        attempts: [attempt(legacy), attempt(legacy)],
      },
      native: {
        state: "refused",
        argv: ["--native"],
        attempts: [attempt(native), attempt(native)],
      },
      comparison: { state: "not-compared", reason: "no-reviewed-complete-oracle" },
    };
  });
  return {
    schema: "vize.focus-history.capture",
    version: 1,
    acceptance: "unreviewed",
    sourceRevision: receipt.source.sourceRevision,
    fixtureSha256: "5ab754f0a882fc9c7f8eb843edf454af7909793debe985c0096daa6b132708ba",
    buildReceipt: receipt,
    rows,
    summary: focusSummary(rows),
  };
}

void test("capture packets reject missing custody, false success, wrong refusal and invented acceptance", () => {
  const valid = syntheticReport();
  validateFocusCapture(fixtures, valid, receipt);
  const mutations = [
    (r: any) => {
      r.schema = "vize.differential.result";
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
    (r: any) => r.rows[0].legacy.attempts.pop(),
    (r: any) => r.rows[0].native.attempts.pop(),
    (r: any) => {
      r.rows[0].legacy.argv = ["--report"];
    },
    (r: any) => {
      r.rows[0].legacy.attempts[0].exitStatus = 1;
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
      r.rows[0].legacy.attempts[0].referenceComparison = { state: "equal" };
    },
    (r: any) => {
      r.rows[0].native.provenance = { scope: "whole-product" };
    },
    (r: any) => {
      r.rows[0].comparison.state = "equal";
    },
    (r: any) => {
      r.summary.acceptedCompleteOracles = 8;
    },
    (r: any) => {
      r.summary.nativeHandled = 8;
    },
    (r: any) => {
      r.sourceRevision = "b".repeat(40);
    },
    (r: any) => {
      r.accepted = true;
    },
    (r: any) => {
      r.rows[0].accepted = true;
    },
    (r: any) => {
      r.rows[0].legacy.accepted = true;
    },
    (r: any) => {
      r.rows[0].legacy.attempts[0].accepted = true;
    },
    (r: any) => {
      const bytes = Buffer.from("not a complete observation\n");
      r.rows[0].legacy.attempts[1] = attempt(bytes);
    },
    (r: any) => {
      for (const raw of r.rows[0].native.attempts) {
        const packet = JSON.parse(Buffer.from(raw.stdoutBase64, "base64").toString());
        packet.refusal.unprovided_rule = "wrong/rule";
        Object.assign(raw, attempt(Buffer.from(`${JSON.stringify(packet)}\n`)));
      }
    },
  ];
  for (const mutate of mutations) {
    const report = structuredClone(valid);
    mutate(report);
    assert.throws(() => validateFocusCapture(fixtures, report, receipt));
  }
});

void test("a failing first attempt still retains two independent invocations and both streams", () => {
  const calls: any[] = [];
  const run: any = (program: string, argv: string[], options: any) => {
    calls.push({ program, argv, input: options.input });
    return {
      stdout: Buffer.from(`failure-${calls.length}\n`),
      stderr: Buffer.from("actual mock stderr\n"),
      status: 1,
      signal: null,
    };
  };
  const attempts = collectFocusAttempts(
    "/synthetic-observer",
    ["--legacy"],
    fixtures[0].input,
    run,
  );
  assert.equal(calls.length, 2);
  assert.deepEqual(
    attempts.map((item) => item.exitStatus),
    [1, 1],
  );
  assert.notEqual(attempts[0].stdoutSha256, attempts[1].stdoutSha256);
  for (const call of calls) assert(call.input.equals(fixtures[0].input));
  const report: any = syntheticReport();
  report.rows[0].legacy = {
    state: "failed",
    argv: ["--legacy"],
    attempts,
    error: "synthetic actual-process failure",
  };
  report.summary = focusSummary(report.rows);
  validateFocusCapture(fixtures, report, receipt);
  assert.equal(report.summary.captureFailures, 1);
  assert.equal(report.summary.acceptedCompleteOracles, 0);
});

void test("a thrown first invocation preserves the exception and still attempts the second process", () => {
  let calls = 0;
  const run: any = () => {
    calls += 1;
    if (calls === 1) throw new Error("first invocation failed before spawning");
    return {
      stdout: Buffer.from("second raw failure\n"),
      stderr: Buffer.from("second stderr\n"),
      status: 2,
      signal: null,
    };
  };
  const attempts = collectFocusAttempts(
    "/synthetic-observer",
    ["--legacy"],
    fixtures[0].input,
    run,
  );
  assert.equal(calls, 2);
  assert.equal(attempts[0].exitStatus, null);
  assert.equal(attempts[0].signal, null);
  assert.equal(attempts[0].stdoutBase64, "");
  assert.equal(attempts[0].stderrBase64, "");
  assert.equal(attempts[0].processError, "Error: first invocation failed before spawning");
  assert.equal(attempts[1].exitStatus, 2);
  assert.equal(Buffer.from(attempts[1].stdoutBase64, "base64").toString(), "second raw failure\n");
  const report: any = syntheticReport();
  report.rows[0].legacy = {
    state: "failed",
    argv: ["--legacy"],
    attempts,
    error: "retained invocation failure",
  };
  report.summary = focusSummary(report.rows);
  validateFocusCapture(fixtures, report, receipt);
  for (const mutate of [
    (raw: any) => {
      raw.exitStatus = "2";
    },
    (raw: any) => {
      raw.signal = false;
    },
    (raw: any) => {
      raw.processError = {};
    },
    (raw: any) => {
      raw.accepted = true;
    },
  ]) {
    const invalid = structuredClone(report);
    mutate(invalid.rows[0].legacy.attempts[0]);
    assert.throws(() => validateFocusCapture(fixtures, invalid, receipt));
  }
});
