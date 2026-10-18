import assert from "node:assert/strict";
import test from "node:test";
import { mkdtemp, mkdir, readFile, rm, symlink, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { physicalPublicBody } from "../../../tools/support/release/public_acceptance/vrt_trusted_gallery.ts";
import {
  hostedPath,
  staticVrtHost,
} from "../../../tools/support/release/public_acceptance/vrt_host.ts";
import {
  evidence,
  inventory,
} from "../../../tools/support/release/public_acceptance/vrt_artifacts.ts";
import {
  literalCaptureLaw,
  literalProjectRetainer,
} from "../../../tools/support/release/public_acceptance/vrt_trusted_ui.ts";
import { literalVariants } from "../../../tools/support/release/public_acceptance/vrt_literal.ts";
import {
  colorPreviewAddress,
  retainColorPreviewFailure,
} from "../../../tools/support/release/public_acceptance/vrt_trusted_color_capture.ts";
import { assertRealPublicLoopback } from "../../../tools/support/release/public_acceptance/vrt_trusted_network.ts";
import {
  assertExpectedBrowserFailures,
  expectedBrowserFailures,
  safeBrowserURL,
  type BrowserFailureRecord,
} from "../../../tools/support/release/public_acceptance/vrt_trusted_failures.ts";

/** Metadata, real filesystem and bounded static loopback HTTP only; no native/browser/tunnel. */
function literalPacket(title: "Controls" | "Keys", phase: "new" | "match") {
  const n = literalVariants[title].length;
  return {
    success: true,
    summary: {
      total: n,
      new: phase === "new" ? n : 0,
      passed: phase === "match" ? n : 0,
      failed: 0,
      errors: 0,
      skipped: 0,
      duration: 0,
    },
    results: literalVariants[title].map(([variantName]) => ({
      artPath: `/owned/${title}.art.vue`,
      variantName: String(variantName),
      viewport: "authored-compact",
      passed: true,
      snapshotPath: "/owned/snapshot.png",
      currentPath: "/owned/current.png",
      ...(phase === "new" ? { isNew: true } : { diffPercentage: 0 }),
      images: {
        snapshot: "/artifacts/00000000-0000-0000-0000-000000000001",
        current: "/artifacts/00000000-0000-0000-0000-000000000002",
      },
    })),
    reports: {
      json: "/artifacts/00000000-0000-0000-0000-000000000003",
      html: "/artifacts/00000000-0000-0000-0000-000000000004",
    },
  };
}
void test("literal model: exact two/three names, own status and full counters", () => {
  for (const title of ["Controls", "Keys"] as const)
    for (const phase of ["new", "match"] as const)
      literalCaptureLaw(literalPacket(title, phase), title, `/owned/${title}.art.vue`, phase);
});
void test("preview failure retains diagnostics and propagates the original refusal", async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), "vize-preview-refusal-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const e = await evidence(path.join(root, "evidence")),
    original = new Error("original preview timeout");
  // A filesystem-backed inspection/refusal control; this does not claim browser execution.
  await writeFile(path.join(root, "inspection.json"), '{"heading":"Left","frames":[]}\n');
  const inspect = async () =>
    JSON.parse(await readFile(path.join(root, "inspection.json"), "utf8"));
  await assert.rejects(
    retainColorPreviewFailure(original, inspect, e, "first"),
    (error) => error === original,
  );
  const retained = JSON.parse(
    await readFile(path.join(e.output, "first/preview-refusal.json"), "utf8"),
  );
  assert.deepEqual(retained, {
    error: { name: original.name, message: original.message, stack: original.stack },
    observation: { page: { heading: "Left", frames: [] } },
  });
  const inspectionError = new Error("inspection refused");
  await assert.rejects(
    retainColorPreviewFailure(
      original,
      async () => {
        throw inspectionError;
      },
      e,
      "second",
    ),
    (error) => error === original,
  );
  assert.deepEqual(
    JSON.parse(await readFile(path.join(e.output, "second/preview-refusal.json"), "utf8")),
    {
      error: retained.error,
      observation: { inspectionError: String(inspectionError) },
    },
  );
  // A real immutable writer collision cannot hide either refusal or become a successful capture.
  await assert.rejects(retainColorPreviewFailure(original, inspect, e, "first"), (error) => {
    assert.ok(error instanceof AggregateError);
    assert.equal(error.errors[0], original);
    assert.equal(error.errors[1].code, "EEXIST");
    return true;
  });
});
void test("preview URL evidence preserves absent, empty and about:blank identities", () => {
  const base = "https://owned.trycloudflare.com/built/gallery/vrt/";
  assert.equal(colorPreviewAddress(null, base), null);
  assert.equal(colorPreviewAddress("", base), "");
  for (const value of [
    "about:blank",
    "/preview/left.html",
    "https://user:private@owned.trycloudflare.com/preview?private=1#private",
  ]) {
    const actual = colorPreviewAddress(value, base);
    assert.ok(actual && "value" in actual);
    assert.deepEqual(actual, {
      value:
        value === "about:blank"
          ? value
          : `https://owned.trycloudflare.com/${value.startsWith("/") ? "preview/left.html" : "preview"}`,
      protocol: value === "about:blank" ? "about:" : "https:",
      query: value.includes("?"),
      fragment: value.includes("#"),
      credentials: value.includes("@"),
    });
  }
  assert.deepEqual(colorPreviewAddress("data:private", base), { protocol: "data:", opaque: true });
  assert.deepEqual(colorPreviewAddress("http://[invalid", base), { invalid: true });
});
void test("literal model refuses wrong count/name/status/viewport/images/duration", () => {
  for (const mutate of [
    (d: ReturnType<typeof literalPacket>) => {
      d.summary.total = 2;
    },
    (d: ReturnType<typeof literalPacket>) => {
      d.results[0].variantName = "toString";
    },
    (d: ReturnType<typeof literalPacket>) => {
      d.results[0].passed = false;
    },
    (d: ReturnType<typeof literalPacket>) => {
      d.results[0].viewport = "authored";
    },
    (d: ReturnType<typeof literalPacket>) => {
      Reflect.deleteProperty(d.results[0].images, "current");
    },
    (d: ReturnType<typeof literalPacket>) => {
      d.summary.duration = -1;
    },
    (d: ReturnType<typeof literalPacket>) => {
      Reflect.set(d.results[0], "isNew", true);
    },
    (d: ReturnType<typeof literalPacket>) => {
      Reflect.set(d.results[0], "diffPercentage", 1);
    },
  ]) {
    const data = literalPacket("Keys", "match");
    mutate(data);
    assert.throws(() => literalCaptureLaw(data, "Keys", "/owned/Keys.art.vue", "match"));
  }
});
void test("CDP model requires public+secure initiator and loopback resource and rejects source requests", () => {
  const publicOrigin = "https://owned.trycloudflare.com",
    endpoint = "http://127.0.0.1:12345";
  const valid = [
    {
      kind: "response" as const,
      requestId: "1",
      origin: publicOrigin,
      resourceIPAddressSpace: "Public",
    },
    {
      kind: "request" as const,
      requestId: "2",
      origin: endpoint,
      initiatorIPAddressSpace: "Public",
      initiatorIsSecureContext: true,
    },
    {
      kind: "response" as const,
      requestId: "2",
      origin: endpoint,
      resourceIPAddressSpace: "Loopback",
    },
  ];
  assertRealPublicLoopback(valid, publicOrigin, endpoint);
  for (let i = 0; i < valid.length; i++)
    assert.throws(() =>
      assertRealPublicLoopback(
        valid.filter((_, n) => n !== i),
        publicOrigin,
        endpoint,
      ),
    );
  const insecure = structuredClone(valid);
  insecure[1].initiatorIsSecureContext = false;
  assert.throws(() => assertRealPublicLoopback(insecure, publicOrigin, endpoint));
  assert.throws(() =>
    assertRealPublicLoopback(
      [...valid, { kind: "request", requestId: "3", pathname: "/src/Keys.art.vue" }],
      publicOrigin,
      endpoint,
    ),
  );
});
function failureModel() {
  const endpoint = "http://127.0.0.1:12345",
    blob = "blob:https://owned.trycloudflare.com/owned-image";
  const expected = expectedBrowserFailures();
  expected.deniedSession(endpoint);
  expected.capture(endpoint, 400);
  expected.capture(endpoint, 401);
  expected.revokedBlobs([blob], "actual-401");
  const observed: BrowserFailureRecord[] = [
    {
      kind: "failure",
      requestId: "deny",
      phase: "denied-loopback",
      url: `${endpoint}/session`,
      method: "GET",
    },
    {
      kind: "response",
      requestId: "bad",
      phase: "actual-400",
      url: `${endpoint}/capture`,
      method: "POST",
      statusCode: 400,
    },
    {
      kind: "response",
      requestId: "unauthorized",
      phase: "actual-401",
      url: `${endpoint}/capture`,
      method: "POST",
      statusCode: 401,
    },
    { kind: "failure", requestId: "revoked", phase: "actual-401", url: blob, method: "GET" },
  ];
  return { endpoint, blob, expected, observed };
}
void test("failure model: exact denied-session/capture status/revoked-Blob joins", () => {
  const { endpoint, blob, expected, observed } = failureModel();
  assertExpectedBrowserFailures(observed, expected, [
    {
      phase: "actual-400",
      text: "Failed to load resource: the server responded with a status of 400 (Bad Request)",
      url: `${endpoint}/capture`,
      statusCode: 400,
    },
    { phase: "actual-401", text: "Failed to load resource: net::ERR_FILE_NOT_FOUND", url: blob },
  ]);
});
void test("failure model refuses unrelated404, wrong status/URL/phase/method and unowned Blob", () => {
  for (const extra of [
    {
      kind: "response",
      requestId: "extra",
      phase: "actual-400",
      url: "https://owned.trycloudflare.com/unrelated.js",
      statusCode: 404,
    },
    {
      kind: "response",
      requestId: "extra",
      phase: "actual-400",
      url: "http://127.0.0.1:12345/capture",
      method: "POST",
      statusCode: 404,
    },
    {
      kind: "failure",
      requestId: "extra",
      phase: "actual-401",
      url: "blob:https://owned.trycloudflare.com/unowned",
      method: "GET",
    },
    {
      kind: "failure",
      requestId: "extra",
      phase: "denied-loopback",
      url: "http://127.0.0.1:12345/other",
    },
    { kind: "failure", requestId: "extra", phase: "denied-loopback" },
  ] as BrowserFailureRecord[]) {
    const { expected, observed } = failureModel();
    assert.throws(() => assertExpectedBrowserFailures([...observed, extra], expected, []));
  }
  for (const field of ["phase", "url", "method"] as const) {
    const { expected, observed } = failureModel();
    observed[1][field] = "foreign";
    assert.throws(() => assertExpectedBrowserFailures(observed, expected, []));
  }
});
void test("failure model refuses every unregistered 3xx on public or owned loopback URLs", () => {
  for (let statusCode = 300; statusCode < 400; statusCode++)
    for (const url of [
      "https://owned.trycloudflare.com/redirect",
      "http://127.0.0.1:12345/session",
    ]) {
      const { expected, observed } = failureModel();
      const redirect: BrowserFailureRecord = {
        kind: "response",
        requestId: "redirect",
        phase: "denied-loopback",
        method: "GET",
        url,
        statusCode,
      };
      assert.throws(() => assertExpectedBrowserFailures([...observed, redirect], expected, []));
    }
});
void test("console without corresponding exact actual failure/status cannot pass", () => {
  const { expected, observed } = failureModel();
  for (const message of [
    { phase: "actual-400", text: "Failed to load resource: 404", statusCode: 404 },
    {
      phase: "actual-400",
      text: "Failed to load resource: 400",
      statusCode: 400,
      url: "https://owned.trycloudflare.com/unrelated.js",
    },
    { phase: "actual-400", text: "TypeError: groups.push" },
    { phase: "A-Keys-match", text: "Failed to load resource" },
  ])
    assert.throws(() => assertExpectedBrowserFailures(observed, expected, [message]));
  assert.throws(() => assertExpectedBrowserFailures(observed.slice(1), expected, []));
});
void test("URL metadata removes query/userinfo/hash and refuses credential-bearing opaque URLs", () => {
  assert.equal(
    safeBrowserURL("https://user:pass@owned.invalid/file.js?token=private#secret"),
    "https://owned.invalid/file.js",
  );
  assert.equal(
    safeBrowserURL("blob:https://owned.invalid/image"),
    "blob:https://owned.invalid/image",
  );
  for (const value of [
    "data:text/plain,private",
    "blob:https://user:pass@owned.invalid/image",
    "blob:https://owned.invalid/image?token=private",
  ])
    assert.equal(safeBrowserURL(value), undefined);
});
void test("real filesystem: pre-registered retainer keeps whole partial files after natural ENOENT", async (t) => {
  const temporary = await mkdtemp(path.join(os.tmpdir(), "vrt-caller-input-controls-"));
  t.after(() => rm(temporary, { recursive: true, force: true }));
  const project = path.join(temporary, "project"),
    e = await evidence(path.join(temporary, "evidence"));
  const retain = literalProjectRetainer(project, e);
  await retain();
  assert.equal(e.records.length, 0);
  await mkdir(project);
  const bytes = Buffer.from("partial source-only fixture\n");
  await writeFile(path.join(project, "partial.txt"), bytes);
  await assert.rejects(readFile(path.join(project, "missing")), { code: "ENOENT" });
  await retain();
  assert.deepEqual(
    await readFile(path.join(e.output, "trusted-literal/physical-project/partial.txt")),
    bytes,
  );
  const inventory = JSON.parse(
    await readFile(path.join(e.output, "trusted-literal/physical-project-inventory.json"), "utf8"),
  ) as Array<{ path: string; size: string }>;
  assert.equal(inventory.length, 1);
  assert.equal(inventory[0].path, "partial.txt");
  assert.equal(inventory[0].size, String(bytes.length));
});

void test("real emitted-file lookup joins normal assets and the owned gallery document", async (t) => {
  const temporary = await mkdtemp(path.join(os.tmpdir(), "vrt-public-file-controls-"));
  t.after(() => rm(temporary, { recursive: true, force: true }));
  const directory = path.join(temporary, "dist"),
    origin = "https://owned.trycloudflare.com";
  await mkdir(path.join(directory, "gallery/vrt"), { recursive: true });
  const bytes = Buffer.from("whole emitted bytes\n");
  await writeFile(path.join(directory, "asset.js"), bytes);
  await writeFile(path.join(directory, "gallery/vrt/index.html"), bytes);
  for (const relative of ["asset.js", "gallery/vrt/", "gallery/vrt/art-route"])
    assert.deepEqual(
      (await physicalPublicBody({ directory }, `${origin}/built/${relative}`, origin)).bytes,
      bytes,
    );
});
void test("real emitted-file lookup refuses foreign/query/outside-prefix/missing-asset/symlink escapes", async (t) => {
  const temporary = await mkdtemp(path.join(os.tmpdir(), "vrt-public-file-refusals-"));
  t.after(() => rm(temporary, { recursive: true, force: true }));
  const directory = path.join(temporary, "dist"),
    origin = "https://owned.trycloudflare.com";
  await mkdir(directory);
  await writeFile(path.join(temporary, "outside.js"), "outside\n");
  await symlink(path.join(temporary, "outside.js"), path.join(directory, "escape.js"));
  for (const url of [
    "https://foreign.invalid/built/asset.js",
    `${origin}/built/asset.js?token=private`,
    `${origin}/outside/asset.js`,
    `${origin}/built/missing.js`,
    `${origin}/built/escape.js`,
    `${origin}/built/%2e%2e%2foutside.js`,
  ])
    await assert.rejects(physicalPublicBody({ directory }, url, origin));
});

void test("real static hosts retain repeated whole JS, CSS and font bodies in their own namespaces", async (t) => {
  const temporary = await mkdtemp(path.join(os.tmpdir(), "vrt-host-owned-bodies-"));
  t.after(() => rm(temporary, { recursive: true, force: true }));
  const directory = path.join(temporary, "dist");
  await mkdir(directory);
  const assets = [
    ["index.js", Buffer.from("export const original = 1;\n")],
    ["index.css", Buffer.from(".original { color: green; }\n")],
    ["font.ttf", Buffer.from([0, 1, 0, 0, 0, 4, 0, 64, 0, 0, 0, 0])],
  ] as const;
  for (const [name, bytes] of assets) await writeFile(path.join(directory, name), bytes);
  const e = await evidence(path.join(temporary, "evidence"));
  const hosts = [];
  try {
    for (let index = 0; index < 3; index++) hosts.push(await staticVrtHost(directory, e));
    for (const host of hosts) {
      const origin = host.url.slice(0, -hostedPath.length);
      for (const [name, bytes] of assets) {
        const responses = await Promise.all([1, 2, 3].map(() => fetch(`${origin}/built/${name}`)));
        for (const response of responses) {
          assert.equal(response.status, 200);
          assert.deepEqual(Buffer.from(await response.arrayBuffer()), bytes);
        }
      }
    }
    const files = await inventory(path.join(e.output, "http-bodies"));
    assert.equal(files.length, 9, "three hosts each retain all three complete bodies");
    assert.equal(new Set(files.map((file) => file.path.split("/")[0])).size, 3);
    for (const hostPrefix of new Set(files.map((file) => file.path.split("/")[0]))) {
      const saved = await Promise.all(
        files
          .filter((file) => file.path.startsWith(`${hostPrefix}/`))
          .map((file) => readFile(path.join(e.output, "http-bodies", file.path))),
      );
      for (const [, bytes] of assets)
        assert.equal(saved.filter((body) => body.equals(bytes)).length, 1);
    }
    const before = await inventory(e.output);
    await assert.rejects(e.save(before[0].path, Buffer.from("must not overwrite")), {
      code: "EEXIST",
    });
    assert.deepEqual(await inventory(e.output), before, "immutable evidence policy stays intact");
  } finally {
    await Promise.all(hosts.map((host) => host.close()));
  }
});

void test("concurrent and repeated static responses all refuse an actual failed evidence write", async (t) => {
  const temporary = await mkdtemp(path.join(os.tmpdir(), "vrt-host-write-refusal-"));
  t.after(() => rm(temporary, { recursive: true, force: true }));
  const directory = path.join(temporary, "dist");
  await mkdir(directory);
  await writeFile(path.join(directory, "asset.js"), "whole asset\n");
  const e = await evidence(path.join(temporary, "evidence"));
  let writes = 0;
  const host = await staticVrtHost(directory, {
    ...e,
    async save(name, bytes) {
      writes++;
      // Real filesystem refusal at the immutable target, not a substituted
      // error or response. The second request must await this same rejection.
      await mkdir(path.join(e.output, name));
      await e.save(name, bytes);
    },
  });
  try {
    const origin = host.url.slice(0, -hostedPath.length);
    const responses = await Promise.allSettled(
      [1, 2, 3].map(() => fetch(`${origin}/built/asset.js`)),
    );
    assert.deepEqual(
      responses.map((result) => result.status),
      ["rejected", "rejected", "rejected"],
    );
    assert.equal(writes, 1, "one failed immutable body write remains authoritative");
    assert.deepEqual(host.requests, [], "no request is reported as successfully served");
    assert.deepEqual(
      await inventory(e.output),
      [],
      "the failed target contains no fabricated bytes",
    );
  } finally {
    await host.close();
  }
});
