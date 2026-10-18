import { EventEmitter } from "node:events";
import type { CDPSession } from "playwright";
import { publicAddressCustody } from "../../../tools/support/release/public_acceptance/vrt_trusted_network.ts";
import {
  assertExpectedBrowserFailures,
  expectedBrowserFailures,
} from "../../../tools/support/release/public_acceptance/vrt_trusted_failures.ts";
import assert from "node:assert/strict";
import test from "node:test";
import {
  awaitHostedGallery,
  hostedPath,
} from "../../../tools/support/release/public_acceptance/vrt_host.ts";

// Controlled HTTP/DNS models, not installed/Linux/public qualification. A real public 0.458.0
// CLI exited before announcing on an allocated NXDOMAIN origin; the same CLI and origin passed
// only after 76.8s, with normal HTTPS and the exact emitted bytes. Linux run 39020544433's
// private startup error remains unavailable; its trusted-session failure is not reinterpreted.
const gallery = `https://owned.trycloudflare.com${hostedPath}`;
const manifestURL = new URL("api/static.json", gallery).href;
const emitted = Buffer.from('{"arts":[],"previews":{}}\n');

function response(body: BodyInit = emitted, status = 200, url = manifestURL) {
  const result = new Response(body, { status, headers: { "content-type": "application/json" } });
  Object.defineProperty(result, "url", { value: url });
  return result;
}
function network(code: string) {
  return new TypeError("fetch failed", { cause: Object.assign(new Error("private"), { code }) });
}

void test("public provisioning: DNS refusal cannot launch a session before exact emitted bytes", async (t) => {
  t.mock.timers.enable({ apis: ["setTimeout", "Date"], now: 0 });
  t.mock.method(performance, "now", () => Date.now());
  let requests = 0;
  let launches = 0;
  t.mock.method(globalThis, "fetch", async (url: URL, options: RequestInit) => {
    assert.equal(url.href, manifestURL);
    assert.equal(options.redirect, "error");
    assert.ok(options.signal instanceof AbortSignal);
    assert.equal(options.signal.aborted, false);
    if (++requests === 1) throw network("ENOTFOUND");
    return response();
  });
  const ready = awaitHostedGallery(gallery, emitted).then(() => ++launches);
  await Promise.resolve();
  assert.equal(requests, 1);
  assert.equal(launches, 0);
  t.mock.timers.tick(1_000);
  await ready;
  assert.equal(requests, 2);
  assert.equal(launches, 1);
});

void test("public provisioning has a finite deadline and never launches an unresolved origin", async (t) => {
  t.mock.timers.enable({ apis: ["setTimeout", "Date"], now: 0 });
  t.mock.method(performance, "now", () => Date.now());
  let requests = 0;
  t.mock.method(globalThis, "fetch", async () => {
    ++requests;
    throw network("ENOTFOUND");
  });
  const ready = awaitHostedGallery(gallery, emitted);
  const refused = assert.rejects(ready, /provisioning deadline/);
  await Promise.resolve();
  t.mock.timers.tick(90_000);
  await refused;
  assert.equal(requests, 1);
});

void test("only explicit DNS/connect provisioning failures may be retried", async (t) => {
  for (const code of [
    "ENOTFOUND",
    "EAI_AGAIN",
    "ECONNREFUSED",
    "ECONNRESET",
    "ETIMEDOUT",
    "UND_ERR_CONNECT_TIMEOUT",
  ]) {
    await t.test(code, async (s) => {
      s.mock.timers.enable({ apis: ["setTimeout", "Date"], now: 0 });
      s.mock.method(performance, "now", () => Date.now());
      let requests = 0;
      s.mock.method(globalThis, "fetch", async () => {
        if (++requests === 1) throw network(code);
        return response();
      });
      const ready = awaitHostedGallery(gallery, emitted);
      await Promise.resolve();
      s.mock.timers.tick(1_000);
      await ready;
      assert.equal(requests, 2);
    });
  }
});

void test("TLS, redirects, aborts, arbitrary errors and malformed network causes are never retried", async (t) => {
  for (const error of [
    network("UNABLE_TO_VERIFY_LEAF_SIGNATURE"),
    network("CERT_HAS_EXPIRED"),
    network("ERR_TLS_CERT_ALTNAME_INVALID"),
    network("ERR_REDIRECT"),
    new DOMException("private", "TimeoutError"),
    new DOMException("private", "AbortError"),
    new Error("private"),
    new TypeError("other", { cause: Object.assign(new Error(), { code: "ENOTFOUND" }) }),
    new TypeError("fetch failed", { cause: { code: "ENOTFOUND" } }),
  ]) {
    await t.test(error.name, async (s) => {
      let requests = 0;
      s.mock.method(globalThis, "fetch", async () => {
        ++requests;
        throw error;
      });
      await assert.rejects(awaitHostedGallery(gallery, emitted), /HTTPS acquisition refused/);
      assert.equal(requests, 1);
    });
  }
});

void test("foreign or noncanonical public URLs and oversized local manifests cannot initiate fetch", async (t) => {
  let requests = 0;
  t.mock.method(globalThis, "fetch", async () => {
    ++requests;
    return response();
  });
  for (const url of [
    gallery.replace("https:", "http:"),
    gallery.replace("owned.trycloudflare.com", "example.com"),
    gallery.replace("owned.trycloudflare.com", "api.trycloudflare.com"),
    gallery.replace("owned.trycloudflare.com", "owned.trycloudflare.com.evil.invalid"),
    gallery.replace("owned.trycloudflare.com", "user@owned.trycloudflare.com"),
    gallery.replace("owned.trycloudflare.com", "owned.trycloudflare.com:8443"),
    gallery + "?private=1",
    gallery + "#private",
    gallery.slice(0, -1),
    gallery.replace("/built/", "/other/"),
  ])
    await assert.rejects(awaitHostedGallery(url, emitted));
  for (const bytes of [Buffer.alloc(0), Buffer.alloc(262_145)])
    await assert.rejects(awaitHostedGallery(gallery, bytes), /Bounded emitted manifest/);
  assert.equal(requests, 0);
});

void test("wrong status, MIME, response identity and any body mutation are refused without retry", async (t) => {
  for (const build of [
    () => response(emitted, 404),
    () => response(emitted, 502),
    () => response(emitted, 503),
    () => response(emitted, 200, manifestURL.replace("owned.", "foreign.")),
    () => {
      const r = response();
      r.headers.set("content-type", "text/html");
      return r;
    },
    () => response(Buffer.from(emitted.toString().replace('"arts"', '"fake"'))),
    () => response(emitted.subarray(0, -1)),
    () => response(Buffer.concat([emitted, Buffer.from(" ")])),
  ]) {
    await t.test("untrusted response", async (s) => {
      let requests = 0;
      s.mock.method(globalThis, "fetch", async () => {
        ++requests;
        return build();
      });
      await assert.rejects(awaitHostedGallery(gallery, emitted));
      assert.equal(requests, 1);
    });
  }
});

void test("streaming exact bytes work across chunk boundaries; oversized streams are canceled", async (t) => {
  await t.test("exact stream", async (s) => {
    const body = new ReadableStream<Uint8Array>({
      start(c) {
        c.enqueue(emitted.subarray(0, 3));
        c.enqueue(emitted.subarray(3));
        c.close();
      },
    });
    s.mock.method(globalThis, "fetch", async () => response(body));
    await awaitHostedGallery(gallery, emitted);
  });
  await t.test("oversized stream", async (s) => {
    let canceled = false;
    const body = new ReadableStream<Uint8Array>({
      start(c) {
        c.enqueue(Buffer.alloc(emitted.length + 1));
      },
      cancel() {
        canceled = true;
      },
    });
    s.mock.method(globalThis, "fetch", async () => response(body));
    await assert.rejects(awaitHostedGallery(gallery, emitted), /body exceeds emitted/);
    assert.equal(canceled, true);
    assert.equal(body.locked, false);
  });
});

void test("a valid body arriving after the provisioning deadline cannot authorize launch", async (t) => {
  t.mock.timers.enable({ apis: ["Date"], now: 0 });
  t.mock.method(performance, "now", () => Date.now());
  t.mock.method(globalThis, "fetch", async () => {
    t.mock.timers.setTime(90_000);
    return response();
  });
  await assert.rejects(awaitHostedGallery(gallery, emitted), /provisioning deadline/);
});

test("original failed-load fields and request/event phases survive without granting ownership", async () => {
  // Inert CDP transport only; no browser/network failure or product response is substituted.
  const wire = Object.assign(new EventEmitter(), { async send() {} });
  let phase = "before-request";
  const custody = await publicAddressCustody(wire as unknown as CDPSession, () => phase);
  const failed = {
    requestId: "same",
    errorText: "net::ERR_FAILED",
    canceled: false,
    blockedReason: "other",
    type: "Fetch",
    timestamp: 12.5,
    corsErrorStatus: { corsError: "InvalidResponse", failedParameter: "original" },
  };
  wire.emit("Network.loadingFailed", failed);
  phase = "authored-request";
  wire.emit("Network.requestWillBeSent", {
    requestId: "same",
    frameId: "frame",
    loaderId: "loader",
    type: "Document",
    timestamp: 11.5,
    documentURL: "https://authored.example/art?secret=removed",
    request: { url: "https://authored.example/preview?secret=removed", method: "GET" },
  });
  phase = "later-control";
  const second = { requestId: "same", type: "Fetch", timestamp: 13, errorText: "net::ERR_ABORTED" };
  wire.emit("Network.loadingFailed", second);
  const retained = JSON.parse(JSON.stringify(custody.records));
  const common = {
    requestType: "Document",
    frameId: "frame",
    loaderId: "loader",
    requestTimestamp: 11.5,
    documentURL: "https://authored.example/art",
    phase: "authored-request",
    method: "GET",
    origin: "https://authored.example",
    pathname: "/preview",
    url: "https://authored.example/preview",
  };
  const expected = [failed, second].map(({ type, ...original }, index) => ({
    ...common,
    ...original,
    kind: "failure",
    resourceType: type,
    eventPhase: index === 0 ? "before-request" : "later-control",
  }));
  assert.deepEqual(retained, expected);
  assert.equal(Reflect.get(retained[0], "canceled"), false);
  assert.equal(Object.hasOwn(retained[1], "canceled"), false);
  for (const record of custody.records)
    assert.throws(
      () => assertExpectedBrowserFailures([record], expectedBrowserFailures(), []),
      /Unowned browser failure/,
    );
  await custody.close();
  assert.equal(wire.eventNames().length, 0);
});
