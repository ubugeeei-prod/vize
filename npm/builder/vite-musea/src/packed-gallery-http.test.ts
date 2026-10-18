import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test, { type TestContext } from "node:test";
import {
  createPackedGalleryHttpObserver,
  drainPackedGalleryHttp,
  type PackedGalleryHttpRecord,
} from "./packed-gallery-http.ts";

const origin = "http://127.0.0.1:3210";
const url = `${origin}/__musea__/preview/original.html`;
const body = Buffer.from("<button>Original preview</button>");
const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

function deferred<T>() {
  let fulfill!: (value: T) => void;
  const promise = new Promise<T>((resolve) => {
    fulfill = resolve;
  });
  return { promise, resolve: (value: T) => fulfill(value) };
}

async function fixture(context: TestContext) {
  const caseOutput = await mkdtemp(path.join(os.tmpdir(), "musea-http-order-"));
  context.after(() => rm(caseOutput, { recursive: true, force: true }));
  const responses: PackedGalleryHttpRecord[] = [];
  const pending: Promise<void>[] = [];
  const httpErrors: string[] = [];
  return {
    caseOutput,
    responses,
    pending,
    httpErrors,
    observe: createPackedGalleryHttpObserver({
      origin,
      caseOutput,
      responses,
      pending,
      httpErrors,
      sha256,
    }),
  };
}

void test("original body capture starts before delayed metadata and remains the whole cached owner", async (t) => {
  const capture = await fixture(t);
  const headers = deferred<Record<string, string>>();
  const trace: string[] = [];
  let navigated = false;
  capture.observe({
    url: () => url,
    status: () => 200,
    allHeaders: () => headers.promise,
    request: () => ({ allHeaders: async () => ({}), redirectedTo: () => null }),
    body: async () => {
      assert.equal(navigated, false, "the original document must still own the response");
      trace.push("original-body");
      return body;
    },
  });
  const captured = drainPackedGalleryHttp(capture.pending);
  trace.push("navigate");
  navigated = true;
  headers.resolve({ etag: '"original"', "content-type": "text/html" });
  await captured;
  capture.observe({
    url: () => url,
    status: () => 304,
    allHeaders: async () => ({ etag: '"original"' }),
    request: () => ({
      allHeaders: async () => ({ "if-none-match": '"original"' }),
      redirectedTo: () => null,
    }),
    body: async () => {
      throw new Error("A 304 must use its original authenticated body");
    },
  });
  await drainPackedGalleryHttp(capture.pending);
  assert.deepEqual(capture.httpErrors, []);
  assert.deepEqual(trace, ["original-body", "navigate"]);
  assert.deepEqual(capture.responses, [
    {
      url,
      status: 200,
      headers: { etag: '"original"', "content-type": "text/html" },
      requestHeaders: {},
      bytes: body.length,
      sha256: sha256(body),
    },
    {
      url,
      status: 304,
      headers: { etag: '"original"' },
      requestHeaders: { "if-none-match": '"original"' },
      cacheValidation: { ifNoneMatch: '"original"', responseEtag: '"original"' },
      cachedBody: {
        url,
        status: 200,
        etag: '"original"',
        bytes: body.length,
        sha256: sha256(body),
      },
    },
  ]);
  assert.deepEqual(
    await readFile(path.join(capture.caseOutput, "http-bodies", `${sha256(body)}.bin`)),
    body,
  );
});

void test("navigation before body capture retains the entire original protocol failure", async (t) => {
  const capture = await fixture(t);
  const headers = deferred<Record<string, string>>();
  const navigated = true;
  const message =
    "response.body: Protocol error (Network.getResponseBody): No resource with given identifier found\n" +
    "Response body is not available for a response that was navigated away from. Read response.body() before triggering any navigation.";
  capture.observe({
    url: () => url,
    status: () => 200,
    allHeaders: () => headers.promise,
    request: () => ({ allHeaders: async () => ({}), redirectedTo: () => null }),
    body: async () => {
      if (navigated) throw new Error(message);
      return body;
    },
  });
  let drained = false;
  const finished = drainPackedGalleryHttp(capture.pending).then(() => {
    drained = true;
  });
  await new Promise<void>((resolve) => setImmediate(resolve));
  assert.equal(drained, false, "a failed body must still retain its original metadata");
  headers.resolve({ "content-type": "text/html" });
  await finished;
  assert.deepEqual(capture.httpErrors, [`Error: ${message}`]);
  assert.deepEqual(capture.responses, [
    { url, status: 200, headers: { "content-type": "text/html" }, requestHeaders: {} },
  ]);
});

void test("draining includes responses admitted while earlier bodies are being captured", async () => {
  const registered = deferred<void>();
  const bodyCapture = deferred<void>();
  const pending: Promise<void>[] = [];
  pending.push(
    Promise.resolve().then(() => {
      pending.push(bodyCapture.promise);
      registered.resolve();
    }),
  );
  let drained = false;
  const finished = drainPackedGalleryHttp(pending).then(() => {
    drained = true;
  });
  await registered.promise;
  await new Promise<void>((resolve) => setImmediate(resolve));
  assert.equal(drained, false);
  bodyCapture.resolve();
  await finished;
  assert.equal(drained, true);
});

void test("passive body custody records the original completion before delayed metadata", async (t) => {
  const capture = await fixture(t);
  const metadata = deferred<Record<string, string>>();
  const finished = deferred<Buffer>();
  const events: Array<{ event: string; response: unknown; result: unknown }> = [];
  const observe = createPackedGalleryHttpObserver({
    ...capture,
    origin,
    sha256,
    onBodyCapture: (event, response, result) => events.push({ event, response, result }),
  });
  const response = {
    url: () => url,
    status: () => 200,
    allHeaders: () => metadata.promise,
    request: () => ({ allHeaders: async () => ({}), redirectedTo: () => null }),
    body: () => finished.promise,
  };
  observe(response);
  assert.deepEqual(events, [{ event: "body-start", response, result: undefined }]);
  finished.resolve(body);
  await new Promise<void>((resolve) => setImmediate(resolve));
  assert.deepEqual(events, [
    { event: "body-start", response, result: undefined },
    { event: "body-finished", response, result: { bytes: body.length } },
  ]);
  assert.deepEqual(capture.responses, []);
  metadata.resolve({ "content-type": "text/html" });
  await drainPackedGalleryHttp(capture.pending);
  assert.deepEqual(capture.httpErrors, []);
  assert.deepEqual(capture.responses, [
    {
      url,
      status: 200,
      headers: { "content-type": "text/html" },
      requestHeaders: {},
      bytes: body.length,
      sha256: sha256(body),
    },
  ]);
  assert.deepEqual(
    await readFile(path.join(capture.caseOutput, "http-bodies", `${sha256(body)}.bin`)),
    body,
  );
});

void test("passive body custody retains the exact original failure and strict HTTP refusal", async (t) => {
  const capture = await fixture(t);
  const metadata = deferred<Record<string, string>>();
  const failure = new Error("original protocol packet\ncomplete second line", {
    cause: new Error("original underlying cause"),
  });
  Object.assign(failure, { code: "ERR_ORIGINAL_BODY", requestId: "original-id" });
  const events: Array<{ event: string; response: unknown; result: unknown }> = [];
  const observe = createPackedGalleryHttpObserver({
    ...capture,
    origin,
    sha256,
    onBodyCapture: (event, response, result) => events.push({ event, response, result }),
  });
  const response = {
    url: () => url,
    status: () => 200,
    allHeaders: () => metadata.promise,
    request: () => ({ allHeaders: async () => ({}), redirectedTo: () => null }),
    body: async () => {
      throw failure;
    },
  };
  observe(response);
  await new Promise<void>((resolve) => setImmediate(resolve));
  assert.deepEqual(events, [
    { event: "body-start", response, result: undefined },
    { event: "body-error", response, result: failure },
  ]);
  assert.equal(events[1].result, failure);
  assert.deepEqual(capture.httpErrors, []);
  metadata.resolve({ "content-type": "text/html" });
  await drainPackedGalleryHttp(capture.pending);
  assert.deepEqual(capture.httpErrors, [String(failure)]);
  assert.deepEqual(capture.responses, [
    { url, status: 200, headers: { "content-type": "text/html" }, requestHeaders: {} },
  ]);
});
