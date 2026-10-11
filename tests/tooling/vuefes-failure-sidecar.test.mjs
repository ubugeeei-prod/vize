import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { recordFailureTransport } from "../_helpers/vuefes-failure-transport.mjs";
import { writeFailureJson } from "../_helpers/vuefes-failure-json.mjs";
import {
  createVuefesFailureSidecar,
  failureSidecarBudget,
} from "../_helpers/vuefes-failure-sidecar.mjs";

const intent = { schema: "controlled-sidecar-intent; no native build authority", runId: "control" };
const scratch = () => fs.mkdtempSync(path.join(os.tmpdir(), "vuefes-failure-control-"));
const metadata = (outputDir) => ({
  outputDir,
  callbackStartedAt: performance.now(),
  declaredTimeout: 900_000,
  retry: 0,
});

void test("diagnostic budget refuses unknown/expired callback authority without changing test timeout", () => {
  assert.equal(failureSidecarBudget(0, 35_000, 0).milliseconds, 0);
  assert.equal(failureSidecarBudget(0, 35_001, 0).milliseconds, 5_000);
  assert.equal(failureSidecarBudget(0, 1, 2).milliseconds, 0);
  assert.equal(failureSidecarBudget(0, Number.NaN, 0).milliseconds, 0);
  assert.equal(failureSidecarBudget(0, 900_000, 1).reserve, 30_000);
  assert.match(failureSidecarBudget(0, 900_000, 1).authority, /excludes pre-callback/);
});

void test("streamed JSON preserves complete large fields and surrogate boundaries; aborted writes cannot finalize", async () => {
  const output = scratch();
  try {
    const controller = new AbortController();
    const value = {
      entire: `${"x".repeat(16_382)}🧪${'\n"\\'.repeat(100_000)}`,
      nested: [null, false, { full: "tail" }],
    };
    const file = path.join(output, "whole.json");
    await writeFailureJson(file, value, controller.signal);
    assert.deepEqual(JSON.parse(fs.readFileSync(file, "utf8")), value);
    controller.abort();
    await assert.rejects(
      writeFailureJson(path.join(output, "aborted.partial"), value, controller.signal),
    );
    assert.equal(fs.existsSync(path.join(output, "aborted.json")), false);
  } finally {
    fs.rmSync(output, { recursive: true, force: true });
  }
});

void test("deadline quarantines a pending real observer promise and preserves incomplete state after late resolution", async () => {
  const output = scratch();
  const handlers = new Map();
  let finish;
  const late = new Promise((resolve) => {
    finish = resolve;
  });
  const page = {
    on: (name, handler) => handlers.set(name, handler),
    off: (name) => handlers.delete(name),
    evaluate: () => late,
  };
  try {
    const sidecar = createVuefesFailureSidecar(
      [{ page, name: "pending" }],
      metadata(output),
      intent,
    );
    const original = new Error("original failure retained");
    const started = performance.now();
    await sidecar.failed(original);
    assert(performance.now() - started < 6_000);
    assert.equal(handlers.size, 0);
    const statusFile = path.join(output, "retry-0/status.json");
    const before = fs.readFileSync(statusFile);
    assert.equal(JSON.parse(before).complete, false);
    finish({ complete: true });
    await new Promise((resolve) => setImmediate(resolve));
    assert.deepEqual(fs.readFileSync(statusFile), before);
    assert.equal(fs.existsSync(path.join(output, "retry-0/final.json")), false);
  } finally {
    fs.rmSync(output, { recursive: true, force: true });
  }
});

void test("listener and persistence refusal cannot escape into the original test flow", async () => {
  const output = scratch();
  const page = {
    on() {
      throw new Error("listener refused");
    },
    off() {},
    evaluate: async () => ({ complete: true }),
  };
  try {
    const recorder = recordFailureTransport(page);
    recorder.stop();
    assert.match(recorder.freeze().failures[0], /installation refused/);
    const sidecar = createVuefesFailureSidecar(
      [{ page, name: "refused" }],
      metadata(output),
      intent,
    );
    await assert.doesNotReject(
      sidecar.failed(new Error("original"), {
        persist: async () => {
          throw new Error("disk refused");
        },
      }),
    );
    assert.equal(fs.existsSync(path.join(output, "retry-0/final.json")), false);
  } finally {
    fs.rmSync(output, { recursive: true, force: true });
  }
});
