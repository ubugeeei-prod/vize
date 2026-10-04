import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { compareBytes } from "./compare.mjs";
import { sha256 } from "./harness.mjs";

export const NATIVE_APIS = ["--native-current-api", "--native-report", "--native-static-class"];

export function nativeArgv(fixture: any) {
  assert.equal(fixture.argv.length, 1);
  const api = `--native-${fixture.argv[0].slice(2)}`;
  assert(NATIVE_APIS.includes(api));
  return [api];
}

export function validateNativeContract(receipt: any) {
  const [probe] = receipt.probes;
  assert.deepEqual(probe.argv, ["--contract"]);
  const bytes = Buffer.from(probe.stdoutBase64, "base64");
  assert.equal(probe.sha256, sha256(bytes));
  assert.equal(probe.exitStatus, 0);
  assert.deepEqual(JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes)), {
    schema: "vize.linter-history-observer",
    version: 2,
    apis: ["--current-api", "--report", "--static-class"],
    preset: "Incremental",
    locale: "En",
    help: "Full",
    native: "configured-bare-template",
    nativeApis: NATIVE_APIS,
  });
  const nativeProbe = receipt.probes[1];
  assert.deepEqual(nativeProbe.argv, ["--native-contract"]);
  assert.equal(nativeProbe.exitStatus, 0);
  const nativeBytes = Buffer.from(nativeProbe.stdoutBase64, "base64");
  assert.equal(nativeProbe.sha256, sha256(nativeBytes));
  assert.deepEqual(JSON.parse(nativeBytes.toString()), {
    schema: "vize.linter-native-observer",
    version: 1,
    apis: NATIVE_APIS,
    owner: "NativeLintComponent",
    entry: "template",
    wholeOutput: "Case+Observation",
    fallback: false,
  });
  assert.equal(receipt.probes.length, 2);
}

// This pins the immutable pack's expected capability/refusal contract. Actual
// handling still requires source-built normal API execution and full output.
export function expectedNativeReason(fixture: any) {
  const api = nativeArgv(fixture)[0];
  const input = JSON.parse(fixture.input.toString());
  if (api === "--native-report")
    return {
      api,
      entry: null,
      kind: "ApiUnavailable",
      detail: "native whole report API is not provided",
    };
  const entry = input.entry;
  if (entry !== "template")
    return {
      api,
      entry,
      kind: "EntryUnavailable",
      detail: `native original ${entry} entry is not provided`,
    };
  if (input.vue_version === "2")
    return {
      api,
      entry,
      kind: "UnsupportedVueVersion",
      detail: "UnsupportedVueVersion { requested: V2 }",
    };
  if (input.vapor === true)
    return {
      api,
      entry,
      kind: "UnsupportedVaporMode",
      detail: "UnsupportedVaporMode { requested: true }",
    };
  const rule = api === "--native-static-class" ? "vapor/prefer-static-class" : input.rule;
  if (rule !== "vue/component-definition-name-casing")
    return { api, entry, kind: "UnprovidedRule", detail: `UnprovidedRule { rule: "${rule}" }` };
  return null;
}

export function decodeNativeOutcome(bytes: Buffer, fixture: any) {
  assert(bytes.length > 0 && bytes.at(-1) === 10);
  const outcome = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes));
  assert(["handled", "unsupported"].includes(outcome.state));
  if (outcome.state === "handled") {
    assert.equal(
      expectedNativeReason(fixture),
      null,
      "unprovided configured entry cannot claim handled output",
    );
    assert.deepEqual(Object.keys(outcome).sort(), ["observation", "state"]);
    assert.equal(typeof outcome.observation, "string");
    assert(outcome.observation.length > 0 && outcome.observation.endsWith("\n"));
    return outcome;
  }
  assert.deepEqual(Object.keys(outcome).sort(), ["reason", "state"]);
  const expected = expectedNativeReason(fixture);
  assert(expected, "the eight pinned admissible originals require whole output, not refusal");
  assert.deepEqual(
    outcome.reason,
    expected,
    "actual exact authored API/entry/rule refusal required",
  );
  return outcome;
}

function provenance(receipt: any) {
  return {
    scope: "whole-product",
    sourceRevision: receipt.source.sourceRevision,
    buildReceiptSha256: sha256(Buffer.from(JSON.stringify(receipt))),
    contributions: ["parse", "lint", "output"].map((stage) => ({
      stage,
      implementation: "native",
      factOrigin: "native",
      fallback: false,
    })),
  };
}

export function runNative(binaryPath: string, fixture: any, receipt: any) {
  const native: any = {
    state: "failed",
    argv: nativeArgv(fixture),
    inputSha256: sha256(fixture.input),
    attempts: [],
  };
  try {
    let previous: Buffer | undefined;
    let outcome: any;
    for (let pass = 0; pass < 2; pass++) {
      const result = spawnSync(binaryPath, native.argv, {
        input: fixture.input,
        timeout: 30_000,
        maxBuffer: 8 * 1024 * 1024,
      });
      const stdout = result.stdout ?? Buffer.alloc(0);
      const stderr = result.stderr ?? Buffer.alloc(0);
      const attempt: any = {
        stdoutBase64: stdout.toString("base64"),
        stdoutSha256: sha256(stdout),
        stderrBase64: stderr.toString("base64"),
        stderrSha256: sha256(stderr),
        exitStatus: result.status,
        signal: result.signal,
        processError: result.error?.message ?? null,
      };
      native.attempts.push(attempt);
    }
    for (const attempt of native.attempts) {
      const stdout = Buffer.from(attempt.stdoutBase64, "base64");
      const stderr = Buffer.from(attempt.stderrBase64, "base64");
      assert.equal(attempt.processError, null, attempt.processError ?? undefined);
      assert.equal(attempt.signal, null);
      assert.equal(attempt.exitStatus, 0, stderr.toString());
      assert.equal(stderr.length, 0);
      if (previous) assert(stdout.equals(previous), "native complete outcome did not repeat");
      previous = stdout;
      outcome = decodeNativeOutcome(stdout, fixture);
      if (outcome.state === "handled") {
        attempt.referenceComparison = compareBytes(
          fixture.expected,
          Buffer.from(outcome.observation),
        );
      }
    }
    if (outcome.state === "unsupported") {
      native.state = "unsupported";
      native.reason = outcome.reason;
    } else {
      native.state = "completed";
      native.verdict = native.attempts.every(
        (attempt: any) => attempt.referenceComparison.state === "equal",
      )
        ? "matched-reference"
        : "baseline-drift";
      native.provenance = provenance(receipt);
      native.observation = {
        argv: native.argv,
        inputSha256: native.inputSha256,
        attempts: native.attempts,
      };
    }
  } catch (error) {
    native.error = String(error);
  }
  return native;
}

export function validateNativeRow(fixture: any, native: any, receipt: any) {
  assert(["completed", "unsupported", "failed"].includes(native.state));
  assert.deepEqual(native.argv, nativeArgv(fixture));
  assert.equal(native.inputSha256, sha256(fixture.input));
  assert(Array.isArray(native.attempts) && native.attempts.length <= 2);
  let previous: Buffer | undefined;
  const observations: Buffer[] = [];
  for (const attempt of native.attempts) {
    const bytes = Buffer.from(attempt.stdoutBase64, "base64");
    const stderr = Buffer.from(attempt.stderrBase64, "base64");
    assert.equal(attempt.stdoutSha256, sha256(bytes));
    assert.equal(attempt.stderrSha256, sha256(stderr));
    if (native.state === "failed") continue;
    assert.equal(attempt.exitStatus, 0);
    assert.equal(attempt.signal, null);
    assert.equal(attempt.processError, null);
    assert.equal(stderr.length, 0);
    if (previous) assert(bytes.equals(previous), "native raw outcome did not repeat");
    previous = bytes;
    const outcome = decodeNativeOutcome(bytes, fixture);
    if (native.state === "unsupported") {
      assert.equal(outcome.state, "unsupported");
      assert.deepEqual(native.reason, outcome.reason);
      assert.equal(attempt.referenceComparison, undefined);
    } else {
      assert.equal(outcome.state, "handled");
      const observation = Buffer.from(outcome.observation);
      observations.push(observation);
      assert.deepEqual(attempt.referenceComparison, compareBytes(fixture.expected, observation));
    }
  }
  if (native.state === "failed") {
    assert.equal(typeof native.error, "string");
    assert(native.error.length > 0);
    assert.equal(native.provenance, undefined);
  } else {
    assert.equal(native.attempts.length, 2, "both fresh native processes are required");
    assert.equal(native.error, undefined);
    if (native.state === "completed") {
      assert.deepEqual(native.provenance, provenance(receipt));
      assert.deepEqual(native.observation, {
        argv: native.argv,
        inputSha256: native.inputSha256,
        attempts: native.attempts,
      });
      assert.equal(
        native.verdict,
        native.attempts.every((attempt: any) => attempt.referenceComparison.state === "equal")
          ? "matched-reference"
          : "baseline-drift",
      );
    } else {
      assert.equal(native.provenance, undefined);
      assert.equal(native.observation, undefined);
    }
  }
  return observations;
}
