import assert from "node:assert/strict";
import { test } from "node:test";
import { recordFailureTransport } from "../_helpers/vuefes-failure-transport.mjs";

void test("pending response/body cutoff reports its actual phase and freezes against late fulfillment or failure", async () => {
  const handlers = new Map();
  const page = {
    on: (name, handler) => handlers.set(name, handler),
    off: (name) => handlers.delete(name),
  };
  const recorder = recordFailureTransport(page);
  const request = {
    url: () => "http://actual.invalid/root.js",
    method: () => "GET",
    resourceType: () => "script",
    redirectedFrom: () => null,
    failure: () => ({ errorText: "actual request failed" }),
  };
  let finishHeaders;
  let finishBody;
  const headers = new Promise((resolve) => {
    finishHeaders = resolve;
  });
  const body = new Promise((resolve) => {
    finishBody = resolve;
  });
  const response = {
    request: () => request,
    status: () => 200,
    statusText: () => "OK",
    url: request.url,
    fromServiceWorker: () => false,
    headers: () => ({ "content-type": "text/javascript" }),
    allHeaders: () => headers,
    headersArray: async () => [{ name: "Content-Type", value: "text/javascript" }],
    body: () => body,
  };
  handlers.get("request")(request);
  handlers.get("response")(response);
  assert.match(recorder.inventory()[0].missing, /response observed; headers capture pending/);
  finishHeaders({ "content-type": "text/javascript" });
  await new Promise((resolve) => setImmediate(resolve));
  assert.match(recorder.inventory()[0].missing, /headers captured; decoded body pending/);
  assert.equal(recorder.stop(), 1);
  const packet = recorder.freeze();
  assert.equal(packet.pendingAtCutoff, 1);
  assert.equal(packet.complete, false);
  assert.equal(packet.requests[0].response.status, 200);
  const before = JSON.stringify(packet);
  finishBody(Buffer.from("late full body"));
  await recorder.finish();
  assert.equal(JSON.stringify(packet), before);
  assert.equal(handlers.size, 0);

  const again = recordFailureTransport(page);
  handlers.get("response")({
    ...response,
    allHeaders: async () => ({}),
    body: async () => Buffer.from("actual body"),
  });
  await again.finish();
  assert.equal(again.inventory()[0].complete, true);
  handlers.get("requestfailed")(request);
  assert.equal(again.freeze().requests[0].complete, false);
  again.stop();

  const pendingFailure = recordFailureTransport(page);
  let finishFailedBody;
  const failedBody = new Promise((resolve) => {
    finishFailedBody = resolve;
  });
  handlers.get("response")({ ...response, allHeaders: async () => ({}), body: () => failedBody });
  await new Promise((resolve) => setImmediate(resolve));
  handlers.get("requestfailed")(request);
  finishFailedBody(Buffer.from("late bytes for failed request"));
  await pendingFailure.finish();
  const failed = pendingFailure.freeze().requests[0];
  assert.equal(failed.complete, false);
  assert.equal(failed.requestFailure, "actual request failed");
  assert.match(failed.missing, /request failed/);
  assert.equal(failed.body.toString(), "late bytes for failed request");
  pendingFailure.stop();
});
