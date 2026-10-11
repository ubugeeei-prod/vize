import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createServer } from "node:http";
import { EventEmitter } from "node:events";
import type { TestContext } from "node:test";
import { test } from "node:test";
import {
  collectNativeEvent,
  failedBodyAdmission,
  nativeFailureSummary,
  retainFailedBody,
} from "./native-failed-responses.ts";

async function endpoint(
  t: TestContext,
  status: number,
  body: Buffer,
  mode: "complete" | "chunked" | "abort" = "complete",
) {
  const server = createServer((_request, response) => {
    response.writeHead(status, {
      "content-type": "text/plain",
      connection: "close",
      ...(mode === "chunked" ? {} : { "content-length": body.length }),
    });
    if (mode === "abort") {
      response.write(body.subarray(0, 1));
      setImmediate(() => response.destroy());
    } else if (mode === "chunked") {
      response.write(body);
      response.end();
    } else response.end(body);
  });
  t.after(async () => {
    server.closeAllConnections();
    if (server.listening) {
      await new Promise<void>((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      );
    }
  });
  await new Promise<void>((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  assert.ok(address && typeof address !== "string");
  return fetch("http://127.0.0.1:" + address.port);
}

for (const status of [503, 504]) {
  void test(
    "retains the same real HTTP " + status + " body without accepting its status",
    async (t) => {
      const bytes = Buffer.from("Actual diagnostic reason for status " + status + "\n");
      const response = await endpoint(t, status, bytes);
      assert.equal(response.status, status);
      assert.equal(response.status < 400, false);
      const admission = failedBodyAdmission(response.headers.get("content-length"), 64 * 1024);
      assert.equal(admission.bytes, bytes.length);
      const retained = await retainFailedBody(
        { body: async () => Buffer.from(await response.arrayBuffer()) },
        bytes.length,
      );
      assert.deepEqual(Buffer.from(retained.data, "base64"), bytes);
      assert.equal(retained.sha256, createHash("sha256").update(bytes).digest("hex"));
      assert.equal(retained.bytes, bytes.length);
      assert.ok(retained.bodyReadFinishedAt >= retained.bodyReadStartedAt);
    },
  );
}

void test("unknown, oversized and exhausted retained HTTP bodies refuse admission", async (t) => {
  const unknown = await endpoint(t, 504, Buffer.from("chunked reason"), "chunked");
  assert.equal(unknown.status, 504);
  assert.deepEqual(failedBodyAdmission(unknown.headers.get("content-length"), 64 * 1024), {
    refusal: "unknown-content-length",
  });
  await unknown.body?.cancel();
  const large = await endpoint(t, 503, Buffer.alloc(16 * 1024 + 1, 120));
  assert.deepEqual(failedBodyAdmission(large.headers.get("content-length"), 64 * 1024), {
    refusal: "body-limit",
  });
  await large.body?.cancel();
  const exhausted = await endpoint(t, 504, Buffer.from("last reason"));
  assert.deepEqual(failedBodyAdmission(exhausted.headers.get("content-length"), 0), {
    refusal: "retained-body-limit",
  });
  await exhausted.body?.cancel();
  for (const length of ["-1", "1e3", "0x10", "01", "1, 1", "1000000"]) {
    assert.deepEqual(failedBodyAdmission(length, 64 * 1024), { refusal: "invalid-content-length" });
  }
});

void test("an actual aborted HTTP body remains a collection failure", async (t) => {
  const response = await endpoint(t, 504, Buffer.alloc(100, 120), "abort");
  assert.equal(response.status, 504);
  assert.equal(failedBodyAdmission(response.headers.get("content-length"), 64 * 1024).bytes, 100);
  await assert.rejects(
    retainFailedBody({ body: async () => Buffer.from(await response.arrayBuffer()) }, 100),
    TypeError,
  );
});

void test("a mismatched body is rejected after the actual same-response read", async (t) => {
  const response = await endpoint(t, 503, Buffer.from("whole body"));
  assert.equal(response.status < 400, false);
  await assert.rejects(
    retainFailedBody({ body: async () => Buffer.from(await response.arrayBuffer()) }, 1),
    /content-length must match/u,
  );
});

void test("a synchronous additive response failure cannot stop the original HTTP ledger", async (t) => {
  const response = await endpoint(t, 504, Buffer.from("actual failed response"));
  const events = new EventEmitter();
  const errors: { sequence: number | null; stage: string; name: string }[] = [];
  const ledger: number[] = [];
  events.on("response", () =>
    collectNativeEvent(errors, 0, "response-event", () => {
      assert.equal(response.status, 504);
      throw new TypeError("statusText accessor refused; this message is not retained");
    }),
  );
  events.on("response", () => ledger.push(response.status));
  assert.doesNotThrow(() => events.emit("response"));
  assert.deepEqual(ledger, [504]);
  assert.equal(
    ledger.every((status) => status < 400),
    false,
  );
  assert.deepEqual(errors, [{ sequence: 0, stage: "response-event", name: "TypeError" }]);
  await response.body?.cancel();
});

void test("a synchronous request-finished timing failure stays explicit without escaping", () => {
  const events = new EventEmitter();
  const errors: { sequence: number | null; stage: string; name: string }[] = [];
  let followingListener = false;
  let bodyStarted = false;
  const timing: () => number = () => {
    throw new RangeError("timing accessor refused");
  };
  events.on("requestfinished", () =>
    collectNativeEvent(errors, 3, "request-finished-event", () => {
      timing();
      bodyStarted = true;
    }),
  );
  events.on("requestfinished", () => {
    followingListener = true;
  });
  assert.doesNotThrow(() => events.emit("requestfinished"));
  assert.equal(followingListener, true);
  assert.equal(bodyStarted, false);
  assert.deepEqual(errors, [{ sequence: 3, stage: "request-finished-event", name: "RangeError" }]);
});

function summaryCounts(summary: string) {
  const value: unknown = JSON.parse(summary);
  assert.ok(typeof value === "object" && value !== null);
  assert.ok("failures" in value && Array.isArray(value.failures));
  assert.ok("collectionErrors" in value && Array.isArray(value.collectionErrors));
  assert.ok("omitted" in value && typeof value.omitted === "number");
  assert.ok(
    "omittedCollectionErrors" in value && typeof value.omittedCollectionErrors === "number",
  );
  return {
    failures: value.failures.length,
    collectionErrors: value.collectionErrors.length,
    omitted: value.omitted,
    omittedCollectionErrors: value.omittedCollectionErrors,
  };
}

void test("maximum admitted bodies and collection errors keep stdout bounded without losing JSON inputs", () => {
  const bytes = Buffer.alloc(8192, 120);
  const failures = Array.from({ length: 8 }, () => ({
    url: "http://127.0.0.1:3000/" + "x".repeat(234),
    status: 504,
    statusText: "X".repeat(96),
    responseReceivedAt: 1791680000000,
    requestFinishedAt: 1791680000001,
    bodyReadStartedAt: 1791680000002,
    bodyReadFinishedAt: 1791680000003,
    body: {
      bodyReadStartedAt: 1791680000002,
      bodyReadFinishedAt: 1791680000003,
      bytes: bytes.length,
      sha256: createHash("sha256").update(bytes).digest("hex"),
      encoding: "base64",
      data: bytes.toString("base64"),
    },
  }));
  const originalSummary = JSON.stringify({
    failures: failures.map((failure) => ({
      url: failure.url,
      status: failure.status,
      statusText: failure.statusText,
      refusal: null,
      responseReceivedAt: failure.responseReceivedAt,
      requestFinishedAt: failure.requestFinishedAt,
      bodyReadStartedAt: failure.bodyReadStartedAt,
      bodyReadFinishedAt: failure.bodyReadFinishedAt,
      bodyBytes: failure.body.bytes,
      bodySHA256: failure.body.sha256,
      bodyPrefixBase64: bytes.subarray(0, 288).toString("base64"),
    })),
    omitted: 0,
    collectionErrors: [],
  });
  assert.equal(Buffer.byteLength(originalSummary), 8432);
  const before = JSON.stringify(failures);
  const summary = nativeFailureSummary(failures, [], 0);
  assert.ok(Buffer.byteLength("MUSEA_NATIVE_RESPONSE_EVIDENCE " + summary + "\n") <= 8192);
  const shown = summaryCounts(summary);
  assert.equal(shown.failures + shown.omitted, 8);
  assert.ok(shown.omitted > 0);
  assert.equal(shown.omittedCollectionErrors, 0);
  const errors = Array.from({ length: 17 }, (_, sequence) => ({
    sequence,
    stage: "same-finished-response-body",
    name: "E".repeat(64),
  }));
  const second = nativeFailureSummary(failures, errors, 8);
  assert.ok(Buffer.byteLength("MUSEA_NATIVE_RESPONSE_EVIDENCE " + second + "\n") <= 8192);
  const shownSecond = summaryCounts(second);
  assert.equal(shownSecond.failures + shownSecond.omitted, 16);
  assert.equal(shownSecond.collectionErrors + shownSecond.omittedCollectionErrors, 17);
  assert.ok(shownSecond.omittedCollectionErrors > 0);
  assert.equal(JSON.stringify(failures), before);
  assert.equal(
    failures.reduce((total, failure) => total + failure.body.bytes, 0),
    64 * 1024,
  );
});
